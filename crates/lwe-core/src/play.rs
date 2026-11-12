//! Play-layer transitions: exploration, conversation outcomes, investigation,
//! trade, magic and turn-based encounters. These are reducers over the same
//! world state as the simulation — a campaign cannot do anything the world's
//! ledgers, geography and knowledge model do not allow.

use crate::model::*;
use crate::rng::hash_str;
use crate::rules;
use crate::sim::social::{add_grievance, arrest, kill, record_crime, watch_captain};
use crate::sim::{inc, step_day, Incident};
use serde_json::{json, Value};

fn gs<'a>(p: &'a Value, k: &str) -> Res<&'a str> {
    p.get(k).and_then(|v| v.as_str()).ok_or(Reject { code: "invalid_payload".into(), message: format!("missing string field '{k}'") })
}
fn gi(p: &Value, k: &str) -> Res<i64> {
    p.get(k).and_then(|v| v.as_i64()).ok_or(Reject { code: "invalid_payload".into(), message: format!("missing integer field '{k}'") })
}

/// Advance the party clock; crossing midnight runs the full world step.
pub fn advance_minutes(s: &mut WorldState, minutes: i64, out: &mut Vec<Incident>) {
    s.party.minute += minutes.max(0);
    while s.party.minute >= 1440 {
        s.party.minute -= 1440;
        out.extend(step_day(s));
    }
}

pub fn slot(minute: i64) -> &'static str {
    match minute / 60 {
        0..=5 => "night",
        6..=17 => "day",
        18..=21 => "evening",
        _ => "night",
    }
}

/// Where is this person right now? Derived from routine, not stored.
pub fn whereabouts(s: &WorldState, p: &Person) -> Id {
    if p.jailed_until.is_some() {
        return "loc_gaol".into();
    }
    if p.is_pc {
        return s.party.location.clone();
    }
    match slot(s.party.minute) {
        "day" => {
            if matches!(p.occupation.as_str(), "child" | "elder") {
                p.home.clone()
            } else {
                p.work.clone()
            }
        }
        "evening" => {
            if p.age(s.meta.day) >= 16 && p.traits.sociability > 50 && (hash_str(&p.id) ^ s.meta.day as u64) % 3 == 0 {
                "loc_tavern".into()
            } else if p.traits.piety > 75 && (hash_str(&p.id) ^ s.meta.day as u64) % 4 == 1 {
                "loc_shrine".into()
            } else {
                p.home.clone()
            }
        }
        _ => p.home.clone(),
    }
}

pub fn present(s: &WorldState, loc: &str) -> Vec<Id> {
    let mut v: Vec<&Person> = s.living().filter(|p| !p.is_pc && whereabouts(s, p) == loc).collect();
    v.sort_by_key(|p| (std::cmp::Reverse(p.tier), p.id.clone()));
    v.into_iter().map(|p| p.id.clone()).collect()
}

fn lead_pc(s: &WorldState) -> Res<Id> {
    match s.party.members.iter().find(|m| s.persons.get(*m).map(|p| p.alive()).unwrap_or(false)) {
        Some(m) => Ok(m.clone()),
        None => reject("no_party", "create a player character first"),
    }
}

fn journal(s: &mut WorldState, kind: &str, text: String) {
    let day = s.meta.day;
    s.party.journal.push(JournalEntry { day, kind: kind.into(), text });
    if s.party.journal.len() > 300 {
        s.party.journal.remove(0);
    }
}

fn guard_free(s: &WorldState) -> Res<()> {
    if s.encounter.as_ref().map(|e| e.active).unwrap_or(false) {
        return reject("in_combat", "resolve the current encounter first");
    }
    let pc = lead_pc(s)?;
    if s.persons[&pc].jailed_until.is_some() {
        return reject("in_gaol", "your character is in the Stone Cellar; wait out the sentence");
    }
    Ok(())
}

fn party_hh(s: &WorldState) -> Res<Id> {
    Ok(s.persons[&lead_pc(s)?].household.clone())
}

fn stats_of(p: &Person, side: &str) -> Combatant {
    Combatant { id: p.id.clone(), name: p.full_name(), side: side.into(), hp: p.stats.hp, max_hp: p.stats.max_hp, ac: p.stats.ac, init: 0, attack_bonus: p.stats.attack_bonus, dmg_dice: p.stats.dmg_dice, dmg_sides: p.stats.dmg_sides, dmg_bonus: p.stats.dmg_bonus, dodging: false, fled: false, down: false, surprised: false, is_pc: p.is_pc, courage: p.traits.courage }
}

fn start_encounter(s: &mut WorldState, foes: &[Id], nonlethal: bool, surprised_side: &str, ambush: bool, out: &mut Vec<Incident>) -> Res<()> {
    let prof = rules::profile(s);
    let mut cs: Vec<Combatant> = vec![];
    for m in s.party.members.clone() {
        let p = &s.persons[&m];
        if p.alive() && p.stats.hp > 0 {
            cs.push(stats_of(p, "party"));
        }
    }
    for f in foes {
        cs.push(stats_of(&s.persons[f], "foes"));
    }
    for c in cs.iter_mut() {
        let dex = rules::ability_mod(s.persons[&c.id].stats.abilities[1]);
        let surprised = c.side == surprised_side;
        // 2024: surprise gives Disadvantage on Initiative. 2014: surprised creatures cannot act in round 1.
        let adv = if surprised && prof == "srd-5.2.1" { -1 } else { 0 };
        c.surprised = surprised && prof == "srd-5.1";
        let r = rules::d20(s, dex, adv);
        c.init = r.total * 100 + dex + 50;
    }
    cs.sort_by_key(|c| (std::cmp::Reverse(c.init), c.id.clone()));
    let names: Vec<String> = foes.iter().map(|f| s.person_name(f)).collect();
    let mut log = vec![format!("Initiative ({prof}): {}", cs.iter().map(|c| format!("{} {}", c.name, c.init / 100)).collect::<Vec<_>>().join(", "))];
    if ambush {
        log.insert(0, format!("{} steps out of the crowd — this has been coming.", names.join(" and ")));
    }
    s.encounter = Some(Encounter { active: true, round: 1, turn: 0, combatants: cs, log, nonlethal, outcome: String::new(), location: s.party.location.clone() });
    inc(out, "combat.began", true, format!("Blades are out at {}: the party against {}.", s.locations[&s.party.location].name, names.join(", ")), json!({"foes": foes, "ambush": ambush, "rules": prof}));
    run_npc_turns(s, out);
    Ok(())
}

fn alive_side(e: &Encounter, side: &str) -> Vec<usize> {
    e.combatants.iter().enumerate().filter(|(_, c)| c.side == side && !c.down && !c.fled).map(|(i, _)| i).collect()
}

fn next_turn(e: &mut Encounter) {
    loop {
        e.turn += 1;
        if e.turn >= e.combatants.len() {
            e.turn = 0;
            e.round += 1;
            for c in e.combatants.iter_mut() {
                c.surprised = false;
            }
        }
        let c = &mut e.combatants[e.turn];
        if !c.down && !c.fled {
            c.dodging = false;
            break;
        }
    }
}

fn do_attack(s: &mut WorldState, ai: usize, ti: usize) {
    let mut e = s.encounter.take().unwrap();
    let (a, t) = (e.combatants[ai].clone(), e.combatants[ti].clone());
    let prof = rules::profile(s);
    let (flat, shift) = rules::exhaustion_effect(&prof, s.persons[&a.id].stats.exhaustion, "attack");
    let adv = shift + if t.dodging { -1 } else { 0 };
    let r = rules::attack(s, a.attack_bonus + flat, t.ac, a.dmg_dice, a.dmg_sides, a.dmg_bonus, adv);
    let mut line = format!("{} attacks {}: d20 {:?}{:+} = {} vs AC {}", a.name, t.name, r.roll.rolls, r.roll.modifier, r.roll.total, t.ac);
    if r.hit {
        let tc = &mut e.combatants[ti];
        tc.hp -= r.damage;
        line += &format!(" — {} for {} damage", if r.crit { "CRITICAL HIT" } else { "hit" }, r.damage);
        if tc.hp <= 0 {
            tc.hp = 0;
            tc.down = true;
            line += &format!("; {} goes down", t.name);
        }
    } else {
        line += " — miss";
    }
    e.log.push(line);
    s.encounter = Some(e);
}

/// NPCs act until it is a PC's turn or the fight ends.
fn run_npc_turns(s: &mut WorldState, out: &mut Vec<Incident>) {
    for _ in 0..200 {
        let Some(e) = s.encounter.clone() else { return };
        if !e.active {
            return;
        }
        let party = alive_side(&e, "party");
        let foes = alive_side(&e, "foes");
        if party.is_empty() || foes.is_empty() {
            finish(s, out);
            return;
        }
        let c = e.combatants[e.turn].clone();
        if c.down || c.fled {
            next_turn(s.encounter.as_mut().unwrap());
            continue;
        }
        if c.is_pc {
            if c.surprised {
                let en = s.encounter.as_mut().unwrap();
                en.log.push(format!("{} is surprised and cannot act this round.", c.name));
                next_turn(en);
                continue;
            }
            return;
        }
        if c.surprised {
            let en = s.encounter.as_mut().unwrap();
            en.log.push(format!("{} is surprised and cannot act this round.", c.name));
            next_turn(en);
            continue;
        }
        // morale: badly hurt and not brave -> run
        if c.hp * 100 / c.max_hp.max(1) < 35 && c.courage < 60 {
            let en = s.encounter.as_mut().unwrap();
            en.combatants[en.turn].fled = true;
            en.log.push(format!("{} breaks and runs.", c.name));
            next_turn(en);
            continue;
        }
        let target = *party.iter().min_by_key(|i| (e.combatants[**i].hp, e.combatants[**i].id.clone())).unwrap();
        do_attack(s, e.turn, target);
        next_turn(s.encounter.as_mut().unwrap());
    }
}

fn finish(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let mut e = s.encounter.take().unwrap();
    e.active = false;
    let party_up = !alive_side(&e, "party").is_empty();
    e.outcome = if party_up { "victory".into() } else { "defeat".into() };
    let Ok(lead) = lead_pc(s) else {
        s.encounter = Some(e);
        return;
    };
    let lead_name = s.person_name(&lead);
    let loc = e.location.clone();
    for c in e.combatants.clone() {
        if c.is_pc {
            let p = s.persons.get_mut(&c.id).unwrap();
            p.stats.hp = c.hp.max(1); // downed PCs are stabilised: player death is a protected choice (Decision 26)
            continue;
        }
        let hunted = s.persons[&c.id].flags.iter().any(|f| f == "hunting_party");
        if c.down && !e.nonlethal {
            e.log.push(format!("{} is dead.", c.name));
            let kin: Vec<Id> = {
                let p = &s.persons[&c.id];
                let mut k = p.children.clone();
                k.extend(p.parents.clone());
                if let Some(sp) = &p.spouse {
                    k.push(sp.clone());
                }
                k
            };
            if !hunted {
                record_crime(s, "murder", &lead, &c.id, &loc, "", 0, "a fight with the party", out);
            }
            kill(s, &c.id, &format!("killed by {lead_name}"), out);
            for k in kin {
                if s.persons.get(&k).map(|q| q.alive() && q.age(day) >= 14).unwrap_or(false) {
                    add_grievance(s, &k, &lead, &format!("killed {}", c.name), 60, out);
                }
            }
        } else {
            // Survivors remember, and come back harder (nemesis growth).
            let beaten = c.down || c.fled;
            let p = s.persons.get_mut(&c.id).unwrap();
            p.stats.hp = c.hp.max(1);
            if beaten {
                p.health = (p.health - 15).max(5);
                p.flags.push(if c.down { format!("scarred_by:{lead}") } else { format!("fled_from:{lead}") });
                p.stats.level += 1;
                p.stats.max_hp += 5;
                p.stats.attack_bonus += 1;
                p.stats.ac += 1;
                p.remember(day, "defeat", format!("{lead_name} beat me at {}. I will not forget it.", s.locations[&loc].name), 98, vec![lead.clone()]);
                e.log.push(format!("{} survives, and will remember this.", c.name));
            } else {
                p.flags.push(format!("bested:{lead}"));
                p.flags.retain(|f| f != "hunting_party");
                p.stats.level += 1;
                p.stats.max_hp += 6;
                p.stats.attack_bonus += 1;
                p.remember(day, "victory", format!("I put {lead_name} in the dirt at {}.", s.locations[&loc].name), 95, vec![lead.clone()]);
                e.log.push(format!("{} stands over the party, and the whole street saw it.", c.name));
            }
            if p.tier < 3 {
                p.tier = 3;
                p.agent_profile = "t3-major".into();
            }
            if beaten {
                add_grievance(s, &c.id, &lead, if c.down { "beat me down in the street" } else { "made me run like a coward" }, 45, out);
                if !hunted && !e.nonlethal {
                    record_crime(s, "assault", &lead, &c.id, &loc, "", 0, "a fight with the party", out);
                }
            }
        }
    }
    if !party_up {
        let hh = s.persons[&lead].household.clone();
        let lost = s.acct(&hh, "coin") / 2;
        let _ = s.transfer(&hh, "org_reeve", "coin", lost);
        e.log.push(format!("The party wakes at the Shrine of the Patient Water, bandaged and {} marks lighter (the healer's fee and the watch's fine).", lost / M));
        s.party.location = "loc_shrine".into();
        for m in s.party.members.clone() {
            s.persons.get_mut(&m).unwrap().stats.exhaustion += 1;
        }
    }
    journal(s, "combat", format!("Fight at {}: {}.", s.locations[&loc].name, e.outcome));
    inc(out, "combat.ended", true, format!("The fight at {} is over: {} for the party.", s.locations[&loc].name, e.outcome), json!({"outcome": e.outcome, "log": e.log}));
    s.encounter = Some(e);
}

pub fn apply(s: &mut WorldState, kind: &str, p: &Value) -> Res<Vec<Incident>> {
    let mut out = vec![];
    let day = s.meta.day;
    match kind {
        "play.moved" => {
            guard_free(s)?;
            let to = gs(p, "to")?;
            let here = s.locations.get(&s.party.location).cloned().ok_or(Reject { code: "unknown_entity".into(), message: "party is nowhere".into() })?;
            let Some((_, mins)) = here.links.iter().find(|(l, _)| l == to) else {
                return reject("no_path", format!("{} is not directly reachable from {}", to, here.name));
            };
            s.party.location = to.to_string();
            advance_minutes(s, *mins, &mut out);
            // anyone hunting the party who is here and bold enough takes their chance
            let hunters: Vec<Id> = present(s, to).into_iter().filter(|id| {
                let q = &s.persons[id];
                q.flags.iter().any(|f| f == "hunting_party") && q.traits.courage > 35
            }).take(2).collect();
            if !hunters.is_empty() {
                start_encounter(s, &hunters, false, "party", true, &mut out)?;
            }
        }
        "play.waited" => {
            if s.encounter.as_ref().map(|e| e.active).unwrap_or(false) {
                return reject("in_combat", "resolve the current encounter first");
            }
            let m = gi(p, "minutes")?;
            if !(1..=1440 * 30).contains(&m) {
                return reject("invalid_payload", "minutes must be between 1 and 43200");
            }
            advance_minutes(s, m, &mut out);
        }
        "play.rested" => {
            guard_free(s)?;
            advance_minutes(s, 480, &mut out);
            for m in s.party.members.clone() {
                let q = s.persons.get_mut(&m).unwrap();
                q.stats.hp = q.stats.max_hp;
                q.stats.exhaustion = (q.stats.exhaustion - 1).max(0);
            }
            journal(s, "rest", "The party took a long rest.".into());
        }
        "play.talked" => {
            guard_free(s)?;
            let npc = gs(p, "npc")?.to_string();
            let Some(n) = s.persons.get(&npc).cloned() else { return reject("unknown_entity", "no such person") };
            if !n.alive() || whereabouts(s, &n) != s.party.location {
                return reject("not_present", format!("{} is not here", n.full_name()));
            }
            let lead = lead_pc(s)?;
            let said = gs(p, "player_text")?.to_string();
            let reply = gs(p, "npc_text")?.to_string();
            let attitude = p.get("attitude").and_then(|v| v.as_i64()).unwrap_or(0).clamp(-3, 3);
            let disclosed: Vec<String> = p.get("disclosed").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            // Knowledge firewall (AT-011): an NPC can only disclose what that NPC actually knows.
            for c in &disclosed {
                if s.knows(&npc, c).is_none() {
                    return reject("knowledge_leak", format!("{} does not know claim {c}; disclosure refused", n.full_name()));
                }
            }
            for c in &disclosed {
                let conf = s.knows(&npc, c).unwrap_or(50);
                let text = s.claims[c].text.clone();
                if !s.party.knowledge.contains_key(c) {
                    s.party.knowledge.insert(c.clone(), Know { confidence: conf * 8 / 10, day, source: Some(npc.clone()) });
                    journal(s, "testimony", format!("{} told us: {}", n.full_name(), text));
                }
            }
            let lead_name = s.person_name(&lead);
            {
                let r = s.rel_mut(&npc, &lead);
                r.familiarity = (r.familiarity + 4).min(100);
                r.fondness = (r.fondness + attitude * 3).clamp(0, 100);
                r.trust = (r.trust + attitude * 2).clamp(0, 100);
            }
            let q = s.persons.get_mut(&npc).unwrap();
            q.talks_with_party += 1;
            let gist: String = said.chars().take(90).collect();
            q.remember(day, "conversation", format!("Spoke with {lead_name}, who said: \"{gist}\""), 45 + attitude.abs() * 8, vec![lead.clone()]);
            let conv = s.party.conversations.entry(npc.clone()).or_default();
            conv.push(Line { who: "player".into(), text: said, day });
            conv.push(Line { who: "npc".into(), text: reply, day });
            if conv.len() > 16 {
                let cut = conv.len() - 16;
                conv.drain(0..cut);
            }
            advance_minutes(s, 6, &mut out);
        }
        "play.checked" => {
            let pc = p.get("pc").and_then(|v| v.as_str()).map(String::from).unwrap_or(lead_pc(s)?);
            let skill = gs(p, "skill")?;
            let dc = gi(p, "dc")?;
            let adv = p.get("adv").and_then(|v| v.as_i64()).unwrap_or(0);
            let (roll, ok) = rules::skill_check(s, &pc, skill, dc, adv)?;
            inc(&mut out, "dice.rolled", true, format!("{} — {skill} check: d20 {:?}{:+} = {} vs DC {dc}: {}", s.person_name(&pc), roll.rolls, roll.modifier, roll.total, if ok { "success" } else { "failure" }), json!({"pc": pc, "skill": skill, "dc": dc, "roll": roll.json(), "success": ok, "rules": rules::profile(s)}));
        }
        "play.searched" => {
            guard_free(s)?;
            let pc = lead_pc(s)?;
            let loc = s.party.location.clone();
            let evs: Vec<Evidence> = s.evidence.values().filter(|e| e.location == loc && !e.found_by_party && e.quality > 0).cloned().collect();
            let mut found = 0;
            for e in evs {
                let dc = 9 + (100 - e.quality) / 10;
                let (roll, ok) = rules::skill_check(s, &pc, "investigation", dc, 0)?;
                inc(&mut out, "dice.rolled", true, format!("Investigation: d20 {:?}{:+} = {} vs DC {dc}: {}", roll.rolls, roll.modifier, roll.total, if ok { "found something" } else { "nothing" }), json!({"skill": "investigation", "dc": dc, "roll": roll.json(), "success": ok}));
                if ok {
                    found += 1;
                    let ev = s.evidence.get_mut(&e.id).unwrap();
                    ev.found_by_party = true;
                    ev.custody.push(format!("day {day}: examined by the party"));
                    journal(s, "evidence", format!("At {}: {} — {}", s.locations[&loc].name, e.kind, e.desc));
                }
            }
            if found == 0 {
                journal(s, "search", format!("Searched {} and found nothing new.", s.locations[&loc].name));
            }
            advance_minutes(s, 30, &mut out);
        }
        "play.accused" => {
            guard_free(s)?;
            let crime = gs(p, "crime")?.to_string();
            let person = gs(p, "person")?.to_string();
            let Some(c) = s.crimes.get(&crime).cloned() else { return reject("unknown_entity", "no such case") };
            if c.status != "open" {
                return reject("case_closed", "that case is no longer open");
            }
            if !s.persons.get(&person).map(|q| q.alive()).unwrap_or(false) {
                return reject("unknown_entity", "no such person");
            }
            let ev_weight: i64 = s.evidence.values().filter(|e| e.crime == crime && e.found_by_party && e.points_to.as_deref() == Some(person.as_str())).map(|e| e.quality).sum();
            let testimony = s.party.knowledge.keys().filter(|k| s.claims.get(*k).map(|cl| cl.crime.as_deref() == Some(crime.as_str()) && cl.subject.as_deref() == Some(person.as_str())).unwrap_or(false)).count() as i64;
            if ev_weight == 0 && testimony == 0 {
                return reject("no_grounds", "the watch will not act on an accusation with no evidence or testimony behind it");
            }
            let Some(cap) = watch_captain(s) else { return reject("no_watch", "there is no watch-captain to hear the accusation") };
            let add = ev_weight + testimony * 35;
            let total = {
                let cm = s.crimes.get_mut(&crime).unwrap();
                let e = cm.suspicion.entry(person.clone()).or_insert(0);
                *e += add;
                *e
            };
            journal(s, "accusation", format!("We laid what we had against {} before the watch.", s.person_name(&person)));
            let lead = lead_pc(s)?;
            if total >= 100 {
                arrest(s, &crime, &person, &cap, &mut out);
                add_grievance(s, &person, &lead, "handed me to the watch", 30, &mut out);
            }
            advance_minutes(s, 20, &mut out);
        }
        "play.traded" => {
            guard_free(s)?;
            if s.party.location != "loc_market" && s.party.location != "loc_granary" {
                return reject("no_market", "trade happens at Saltmarket Square or the Compact Granary");
            }
            let good = gs(p, "good")?;
            let qty = gi(p, "qty_milli")?;
            let Some(price) = s.market.prices.get(good).copied() else { return reject("unknown_good", "no such commodity") };
            let hh = party_hh(s)?;
            if qty > 0 {
                let cost = qty * price * 108 / 100 / 1000;
                if s.acct("org_compact", good) < qty {
                    return reject("out_of_stock", "the Compact's factor has none to sell");
                }
                s.transfer(&hh, "org_compact", "coin", cost)?;
                s.transfer("org_compact", &hh, good, qty)?;
            } else if qty < 0 {
                let pay = -qty * price * 92 / 100 / 1000;
                if s.acct("org_compact", "coin") < pay {
                    return reject("insufficient_balance", "the factor cannot pay for that much");
                }
                s.transfer(&hh, "org_compact", good, -qty)?;
                s.transfer("org_compact", &hh, "coin", pay)?;
            }
            advance_minutes(s, 10, &mut out);
        }
        "play.gave" => {
            guard_free(s)?;
            let to = gs(p, "to")?.to_string();
            let Some(n) = s.persons.get(&to).cloned() else { return reject("unknown_entity", "no such person") };
            if whereabouts(s, &n) != s.party.location {
                return reject("not_present", "they are not here");
            }
            let commodity = gs(p, "commodity")?;
            let amount = gi(p, "amount_milli")?;
            let hh = party_hh(s)?;
            s.transfer(&hh, &n.household, commodity, amount)?;
            let lead = lead_pc(s)?;
            let lead_name = s.person_name(&lead);
            let r = s.rel_mut(&to, &lead);
            r.fondness = (r.fondness + 8).min(100);
            r.trust = (r.trust + 6).min(100);
            s.persons.get_mut(&to).unwrap().remember(day, "gift", format!("{lead_name} gave us {} {commodity} when we needed it.", amount / M), 75, vec![lead]);
            advance_minutes(s, 5, &mut out);
        }
        "play.stole" => {
            guard_free(s)?;
            let from = gs(p, "from")?.to_string();
            let commodity = gs(p, "commodity")?.to_string();
            let amount = gi(p, "amount_milli")?;
            let seat_ok = s.orgs.get(&from).map(|o| o.seat == s.party.location).unwrap_or(false) || s.households.get(&from).map(|h| h.home == s.party.location).unwrap_or(false);
            if !seat_ok {
                return reject("not_present", "that store is not at this location");
            }
            let pc = lead_pc(s)?;
            let (roll, ok) = rules::skill_check(s, &pc, "stealth", 13, 0)?;
            inc(&mut out, "dice.rolled", true, format!("Stealth: d20 {:?}{:+} = {} vs DC 13: {}", roll.rolls, roll.modifier, roll.total, if ok { "unseen" } else { "noticed" }), json!({"skill": "stealth", "dc": 13, "roll": roll.json(), "success": ok}));
            let hh = party_hh(s)?;
            let loc = s.party.location.clone();
            let taken = s.transfer_up_to(&from, &hh, &commodity, amount);
            let cid = record_crime(s, "theft", &pc, &from, &loc, &commodity, taken, "the party's own purposes", &mut out);
            if !ok {
                // seen in the act: a true sighting enters the town's knowledge
                let text = format!("{} was seen taking {commodity} from {}.", s.person_name(&pc), s.owner_name(&from));
                let cl = s.add_claim(Claim { kind: "sighting".into(), subject: Some(pc.clone()), crime: Some(cid.clone()), text, truth: true, public: true, ..Default::default() });
                *s.crimes.get_mut(&cid).unwrap().suspicion.entry(pc.clone()).or_insert(0) += 70;
                let _ = cl;
            }
            advance_minutes(s, 15, &mut out);
        }
        "play.cast" => {
            guard_free(s)?;
            let pc = p.get("pc").and_then(|v| v.as_str()).map(String::from).unwrap_or(lead_pc(s)?);
            let level = gi(p, "level")?;
            let effect = gs(p, "effect")?;
            let class = s.persons[&pc].stats.class.clone();
            if class != "wizard" && class != "cleric" {
                return reject("not_a_caster", "only trained practitioners can shape potential");
            }
            if !(1..=3).contains(&level) {
                return reject("invalid_payload", "level must be 1..3");
            }
            // Law contract mw-1: Cost(level) = 4·level² thaums, drawn from the ambient field. No overdraft.
            let cost = 4 * level * level * M;
            if s.magic.ambient < cost {
                return reject("insufficient_potential", format!("the ambient field holds {} thaums; this working needs {}", s.magic.ambient / M, cost / M));
            }
            s.magic.ambient -= cost;
            s.magic.dissipated += cost; // bound potential is released as the effect ends; all of it returns via relaxation
            s.magic.spells_cast += 1;
            let wis = rules::ability_mod(s.persons[&pc].stats.abilities[if class == "wizard" { 3 } else { 4 }]);
            match effect {
                "heal" => {
                    let target = p.get("target").and_then(|v| v.as_str()).map(String::from).unwrap_or(pc.clone());
                    let (amt, rolls) = rules::roll_dice(s, level, 8);
                    let Some(t) = s.persons.get_mut(&target) else { return reject("unknown_entity", "no such target") };
                    t.stats.hp = (t.stats.hp + amt + wis).min(t.stats.max_hp);
                    t.health = (t.health + (amt + wis) * 2).min(100);
                    inc(&mut out, "magic.cast", true, format!("{} works a healing: {:?}+{wis} restored. ({} thaums drawn)", s.person_name(&pc), rolls, cost / M), json!({"effect": "heal", "cost_milli": cost, "target": target}));
                }
                "reveal" => {
                    let loc = s.party.location.clone();
                    let ev = s.evidence.values().find(|e| e.location == loc && !e.found_by_party && e.quality > 0).map(|e| e.id.clone());
                    match ev {
                        Some(id) => {
                            let e = s.evidence.get_mut(&id).unwrap();
                            e.found_by_party = true;
                            e.custody.push(format!("day {day}: revealed by a working"));
                            let (k, d) = (e.kind.clone(), e.desc.clone());
                            journal(s, "evidence", format!("A working of revealing showed: {k} — {d}"));
                        }
                        None => journal(s, "search", "The working found nothing hidden here.".into()),
                    }
                    inc(&mut out, "magic.cast", true, format!("{} asks the water what was hidden here. ({} thaums drawn)", s.person_name(&pc), cost / M), json!({"effect": "reveal", "cost_milli": cost}));
                }
                _ => return reject("unknown_effect", "effect must be heal or reveal"),
            }
            advance_minutes(s, 10, &mut out);
        }
        "combat.started" => {
            guard_free(s)?;
            let foes: Vec<Id> = p.get("targets").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            if foes.is_empty() || foes.len() > 4 {
                return reject("invalid_payload", "choose one to four opponents");
            }
            for f in &foes {
                let Some(q) = s.persons.get(f) else { return reject("unknown_entity", "no such person") };
                if !q.alive() || q.is_pc || whereabouts(s, q) != s.party.location {
                    return reject("not_present", format!("{} is not here", q.full_name()));
                }
            }
            let nonlethal = p.get("nonlethal").and_then(|v| v.as_bool()).unwrap_or(false);
            let surprise = p.get("surprise").and_then(|v| v.as_str()).unwrap_or("");
            start_encounter(s, &foes, nonlethal, surprise, false, &mut out)?;
        }
        "combat.acted" => {
            let Some(e) = s.encounter.clone().filter(|e| e.active) else { return reject("no_encounter", "there is no active encounter") };
            let actor = e.combatants[e.turn].clone();
            if !actor.is_pc {
                return reject("not_your_turn", "it is not a player character's turn");
            }
            let action = gs(p, "action")?;
            let prof = rules::profile(s);
            match action {
                "attack" => {
                    let foes = alive_side(&e, "foes");
                    let ti = match p.get("target").and_then(|v| v.as_str()) {
                        Some(t) => *foes.iter().find(|i| e.combatants[**i].id == t).ok_or(Reject { code: "invalid_target".into(), message: "that opponent is not standing".into() })?,
                        None => foes[0],
                    };
                    do_attack(s, e.turn, ti);
                }
                "dodge" => {
                    let en = s.encounter.as_mut().unwrap();
                    en.combatants[en.turn].dodging = true;
                    en.log.push(format!("{} takes the Dodge action.", actor.name));
                }
                "potion" => {
                    if s.persons[&actor.id].stats.potions <= 0 {
                        return reject("no_potion", "no potion of healing left");
                    }
                    let (amt, rolls) = rules::roll_dice(s, 2, 4);
                    s.persons.get_mut(&actor.id).unwrap().stats.potions -= 1;
                    let en = s.encounter.as_mut().unwrap();
                    let c = &mut en.combatants[e.turn];
                    c.hp = (c.hp + amt + 2).min(c.max_hp);
                    en.log.push(format!("{} drinks a potion of healing: {:?}+2 = {} HP ({}).", actor.name, rolls, amt + 2, if rules::potion_uses_action(&prof) { "action, 2014 rules" } else { "bonus action, 2024 rules" }));
                    if !rules::potion_uses_action(&prof) {
                        // 2024: Bonus Action, so the Attack action is still available this turn.
                        let foes = alive_side(s.encounter.as_ref().unwrap(), "foes");
                        if let Some(ti) = foes.first() {
                            do_attack(s, e.turn, *ti);
                        }
                    }
                }
                "flee" => {
                    let en = s.encounter.as_mut().unwrap();
                    for c in en.combatants.iter_mut().filter(|c| c.side == "party") {
                        c.fled = true;
                    }
                    en.log.push("The party breaks off and runs.".into());
                    en.active = false;
                    en.outcome = "fled".into();
                    let ids: Vec<Id> = en.combatants.iter().map(|c| c.id.clone()).collect();
                    let hps: Vec<(Id, i64)> = en.combatants.iter().map(|c| (c.id.clone(), c.hp.max(1))).collect();
                    for (id, hp) in hps {
                        s.persons.get_mut(&id).unwrap().stats.hp = hp;
                    }
                    let lead = lead_pc(s)?;
                    let lead_name = s.person_name(&lead);
                    for id in ids {
                        let q = s.persons.get_mut(&id).unwrap();
                        if !q.is_pc {
                            q.remember(day, "victory", format!("{lead_name} ran from me."), 80, vec![lead.clone()]);
                            q.flags.push(format!("bested:{lead}"));
                        }
                    }
                    inc(&mut out, "combat.ended", true, "The party fled the fight.".into(), json!({"outcome": "fled"}));
                    return Ok(out);
                }
                _ => return reject("invalid_payload", "action must be attack, dodge, potion or flee"),
            }
            next_turn(s.encounter.as_mut().unwrap());
            run_npc_turns(s, &mut out);
            if s.encounter.as_ref().map(|e| e.active).unwrap_or(false) {
                let e2 = s.encounter.as_ref().unwrap();
                if alive_side(e2, "party").is_empty() || alive_side(e2, "foes").is_empty() {
                    finish(s, &mut out);
                }
            }
            if !s.encounter.as_ref().map(|e| e.active).unwrap_or(false) {
                advance_minutes(s, 5, &mut out);
            }
        }
        _ => return reject("unknown_event", format!("unknown play event '{kind}'")),
    }
    Ok(out)
}
