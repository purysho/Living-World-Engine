//! Society: crime and evidence, investigation, daily interaction, rumour,
//! grievance → rivalry → nemesis escalation, organisational politics,
//! courtship and demography. Objective truth (who did it) lives in `Crime`;
//! what people *believe* lives in `Claim`/`knowledge` and may be wrong.

use super::{avg_hunger, can_work, inc, Incident};
use crate::model::*;
use crate::rng::{hash_str, Rng};
use serde_json::json;
use std::collections::BTreeMap;

fn clamp(v: i64, lo: i64, hi: i64) -> i64 {
    v.max(lo).min(hi)
}

fn adjacency(s: &WorldState) -> BTreeMap<Id, Vec<Id>> {
    let mut adj: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    for k in s.relations.keys() {
        let mut it = k.split('|');
        if let (Some(a), Some(b)) = (it.next(), it.next()) {
            adj.entry(a.to_string()).or_default().push(b.to_string());
        }
    }
    adj
}

fn status_rank(st: &str) -> i64 {
    match st {
        "nemesis" => 3,
        "enemy" => 2,
        "rival" => 1,
        _ => 0,
    }
}

/// Recompute the status of edge a->b from affinity and accumulated grievances.
pub fn update_status(s: &mut WorldState, a: &str, b: &str, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let Some(r) = s.relations.get(&rel_key(a, b)) else { return };
    let h = r.hostility(day);
    let maxw = r.grievances.iter().map(|g| g.weight).max().unwrap_or(0);
    let new = if h >= 70 || (h >= 45 && maxw >= 40) {
        "nemesis"
    } else if h >= 35 {
        "enemy"
    } else if h >= 15 {
        "rival"
    } else if r.affinity() >= 75 && r.familiarity >= 60 {
        "ally"
    } else if r.affinity() >= 55 {
        "friend"
    } else {
        "neutral"
    };
    let old = r.status.clone();
    if old == new {
        return;
    }
    let why = r.grievances.last().map(|g| g.what.clone()).unwrap_or_default();
    s.relations.get_mut(&rel_key(a, b)).unwrap().status = new.to_string();
    if status_rank(new) > status_rank(&old) {
        let (an, bn) = (s.person_name(a), s.person_name(b));
        let text = match new {
            "nemesis" => format!("{an} has sworn {bn} a nemesis — {why}."),
            "enemy" => format!("{an} now counts {bn} an enemy — {why}."),
            _ => format!("{an} and {bn} have fallen out — {why}."),
        };
        inc(out, &format!("relation.{new}"), new != "rival", text.clone(), json!({"from": a, "to": b, "status": new, "cause": why}));
        if let Some(p) = s.persons.get_mut(a) {
            p.remember(day, "grudge", text, 60 + status_rank(new) * 12, vec![b.to_string()]);
            if new == "nemesis" && p.tier < 2 {
                p.tier = 2;
                p.agent_profile = "t2-recurring".into();
            }
        }
    }
}

pub fn add_grievance(s: &mut WorldState, from: &str, to: &str, what: &str, weight: i64, out: &mut Vec<Incident>) {
    if from == to || !s.persons.get(from).map(|p| p.alive()).unwrap_or(false) || !s.persons.contains_key(to) {
        return;
    }
    let day = s.meta.day;
    let r = s.rel_mut(from, to);
    r.grievances.push(Grievance { day, what: what.to_string(), weight });
    if r.grievances.len() > 8 {
        let i = r.grievances.iter().enumerate().min_by_key(|(_, g)| g.weight).map(|(i, _)| i).unwrap();
        r.grievances.remove(i);
    }
    r.fondness = clamp(r.fondness - weight / 2, 0, 100);
    r.trust = clamp(r.trust - weight / 2, 0, 100);
    update_status(s, from, to, out);
}

fn kin_of(s: &WorldState, id: &str) -> Vec<Id> {
    let Some(p) = s.persons.get(id) else { return vec![] };
    let mut v: Vec<Id> = p.parents.clone();
    v.extend(p.children.clone());
    if let Some(sp) = &p.spouse {
        v.push(sp.clone());
    }
    v.retain(|k| s.persons.get(k).map(|q| q.alive() && q.age(s.meta.day) >= 14).unwrap_or(false));
    v
}

/// Commit an objective crime with physical evidence and (possibly mistaken) witnesses.
#[allow(clippy::too_many_arguments)]
pub fn record_crime(s: &mut WorldState, kind: &str, culprit: &str, victim: &str, location: &str, commodity: &str, amount: i64, motive: &str, out: &mut Vec<Incident>) -> Id {
    let day = s.meta.day;
    let id = s.next_id("crime");
    let mut r = Rng::new(s.meta.seed, "crimescene", day as u64, hash_str(&id));
    let cname = s.person_name(culprit);
    let lname = s.locations.get(location).map(|l| l.name.clone()).unwrap_or_default();
    let vname = s.owner_name(victim);
    let mut ev_ids = vec![];
    let mut add_ev = |s: &mut WorldState, kind: &str, desc: String, points: Option<Id>, q: i64| {
        let eid = s.next_id("ev");
        s.evidence.insert(eid.clone(), Evidence { id: eid.clone(), crime: id.clone(), kind: kind.into(), location: location.into(), desc, points_to: points, quality: q, found_by_watch: false, found_by_party: false, custody: vec![format!("day {day}: left at {lname}")] });
        ev_ids.push(eid);
    };
    let chome = s.persons.get(culprit).map(|p| p.home.clone()).unwrap_or_default();
    let chome_name = s.locations.get(&chome).map(|l| l.name.clone()).unwrap_or_default();
    match kind {
        "theft" | "sabotage" => {
            if r.chance(650) {
                add_ev(s, "footprints", format!("Boot-prints in the mud, one heel worn down on the outside — matching the gait of {cname}."), Some(culprit.into()), 55);
            }
            if r.chance(300) {
                add_ev(s, "dropped token", format!("A household tally-token dropped in haste, marked for {}.", s.owner_name(&s.persons[culprit].household.clone())), Some(culprit.into()), 75);
            }
            if s.orgs.contains_key(victim) {
                add_ev(s, "forced lock", "The hasp was levered off with a flat bar; whoever did it knew where the watchman would not be.".into(), None, 40);
            }
            add_ev(s, "trail", format!("A scatter of spilled {commodity} leading off toward {chome_name}."), None, 30);
        }
        "embezzlement" => add_ev(s, "altered tally-roll", format!("A tally-roll scraped and re-inked in a hand resembling that of {cname}."), Some(culprit.into()), 50),
        "assault" | "murder" => {
            add_ev(s, "torn cloth", format!("A strip of cloth torn in the struggle, of the kind worn by {cname}."), Some(culprit.into()), 60);
        }
        _ => {}
    }
    // witnesses: anyone living or working at the scene may have seen something — or thinks they did
    let cands: Vec<Person> = s.living().filter(|p| p.id != culprit && !p.is_pc && p.age(day) >= 10 && (p.home == location || p.work == location)).cloned().collect();
    let csex = s.persons[culprit].sex.clone();
    let cage = s.persons[culprit].age(day);
    let lookalikes: Vec<Id> = s.living().filter(|p| p.id != culprit && !p.is_pc && p.sex == csex && (p.age(day) - cage).abs() <= 8).map(|p| p.id.clone()).collect();
    let mut n_wit = 0;
    for w in cands {
        if n_wit >= 4 || !r.chance(40 + w.traits.curiosity / 2) {
            continue;
        }
        n_wit += 1;
        let mistaken = r.chance(250) && !lookalikes.is_empty();
        let seen = if mistaken { r.pick(&lookalikes).clone() } else { culprit.to_string() };
        let text = format!("{} says they saw {} near {} on the night of the {}.", w.full_name(), s.person_name(&seen), lname, kind);
        let c = s.add_claim(Claim { kind: "sighting".into(), subject: Some(seen.clone()), crime: Some(id.clone()), text: text.clone(), truth: !mistaken, origin: Some(w.id.clone()), ..Default::default() });
        s.learn(&w.id, &c, 85, None);
        let seen_name = s.person_name(&seen);
        s.persons.get_mut(&w.id).unwrap().remember(day, "witness", format!("I saw {seen_name} near {lname} the night of the {kind}."), 70, vec![seen]);
    }
    let what = if amount > 0 { format!("{} measures of {commodity}", (amount / M).max(1)) } else { String::new() };
    let notice_text = match kind {
        "theft" => format!("{what} was stolen from {vname} at {lname}."),
        "embezzlement" => format!("Coin is missing from the accounts of {vname}."),
        "sabotage" => format!("Someone wrecked property of {vname} at {lname}."),
        "murder" => format!("{vname} was killed at {lname}."),
        _ => format!("{vname} was attacked at {lname}."),
    };
    let notice = s.add_claim(Claim { kind: "notice".into(), crime: Some(id.clone()), text: notice_text.clone(), truth: true, public: true, ..Default::default() });
    let mut victims: Vec<Id> = vec![];
    if let Some(o) = s.orgs.get(victim) {
        victims.extend(o.members.clone());
    } else if let Some(h) = s.households.get(victim) {
        victims.extend(h.members.clone());
    } else {
        victims.push(victim.to_string());
    }
    for v in victims {
        s.learn(&v, &notice, 95, None);
    }
    s.crimes.insert(id.clone(), Crime { id: id.clone(), day, kind: kind.into(), culprit: culprit.into(), victim: victim.into(), location: location.into(), commodity: commodity.into(), amount, motive: motive.into(), status: "open".into(), accused: None, suspicion: BTreeMap::new(), evidence: ev_ids, closed_day: None });
    if let Some(p) = s.persons.get_mut(culprit) {
        p.remember(day, "crime", format!("I committed {kind} against {vname} ({motive}). No one must know."), 90, vec![victim.to_string()]);
        if p.tier < 2 {
            p.tier = 2;
            p.agent_profile = "t2-recurring".into();
        }
    }
    inc(out, "crime.reported", true, notice_text, json!({"crime": id, "kind": kind, "victim": victim, "location": location}));
    inc(out, "crime.committed", false, format!("[GM] {cname} committed {kind} against {vname} — motive: {motive}. Witnesses: {n_wit}."), json!({"crime": id, "culprit": culprit, "motive": motive}));
    id
}

pub fn crime(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let hhs: Vec<Household> = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty()).cloned().collect();
    for h in &hhs {
        let members: Vec<&Person> = h.members.iter().filter_map(|m| s.persons.get(m)).filter(|p| p.alive()).collect();
        if members.is_empty() {
            continue;
        }
        let hunger = members.iter().map(|p| p.hunger).sum::<i64>() / members.len() as i64;
        if hunger < 42 {
            continue;
        }
        let Some(thief) = members.iter().filter(|p| p.age(day) >= 14 && p.jailed_until.is_none() && !p.is_pc).min_by_key(|p| (p.traits.honesty, p.id.clone())) else { continue };
        let mut r = Rng::new(s.meta.seed, "theft", day as u64, hash_str(&h.id));
        if !r.chance(clamp((hunger - 38) * (110 - thief.traits.honesty) / 45, 1, 90)) {
            continue;
        }
        let need: i64 = members.iter().map(|p| super::food_need(p, day)).sum();
        let thief_id = thief.id.clone();
        let neighbour = hhs.iter().find(|o| o.id != h.id && o.home == h.home && s.acct(&o.id, "grain") > need * 10).map(|o| o.id.clone());
        let (victim, loc) = if s.acct("org_compact", "grain") >= 20 * M && r.chance(550) {
            ("org_compact".to_string(), "loc_granary".to_string())
        } else if s.acct("org_reeve", "grain") >= 20 * M && r.chance(350) {
            ("org_reeve".to_string(), "loc_hall".to_string())
        } else if let Some(n) = neighbour {
            (n, h.home.clone())
        } else if s.acct("hinterland", "grain") >= 20 * M {
            ("hinterland".to_string(), "loc_market".to_string())
        } else {
            continue;
        };
        let amount = s.transfer_up_to(&victim, &h.id, "grain", need * 8);
        if amount > 0 {
            record_crime(s, "theft", &thief_id, &victim, &loc, "grain", amount, "hunger at home", out);
        }
    }
    // greed: embezzlement by the ambitious and unscrupulous
    let cands: Vec<(Id, Id, Id)> = s.persons.values().filter(|p| can_work(p, day) && p.traits.honesty < 25 && p.traits.ambition > 70).filter_map(|p| p.employer.clone().map(|e| (p.id.clone(), e, p.household.clone()))).collect();
    for (pid, org, hh) in cands {
        let mut r = Rng::new(s.meta.seed, "embezzle", day as u64, hash_str(&pid));
        if r.chance_ppm(1100) {
            let seat = s.orgs[&org].seat.clone();
            let amount = s.transfer_up_to(&org, &hh, "coin", r.range(60, 220) * M);
            if amount > 0 {
                record_crime(s, "embezzlement", &pid, &org, &seat, "coin", amount, "ambition and opportunity", out);
            }
        }
    }
}

pub fn watch_captain(s: &WorldState) -> Option<Id> {
    s.persons.values().find(|p| p.alive() && p.jailed_until.is_none() && p.flags.iter().any(|f| f == "watch_captain")).map(|p| p.id.clone())
}

pub fn arrest(s: &mut WorldState, crime_id: &str, accused: &str, by: &str, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let c = s.crimes[crime_id].clone();
    let guilty = c.culprit == accused;
    let (an, bn) = (s.person_name(accused), s.person_name(by));
    {
        let cm = s.crimes.get_mut(crime_id).unwrap();
        cm.status = if guilty { "solved".into() } else { "wrongful".into() };
        cm.accused = Some(accused.to_string());
        cm.closed_day = Some(day);
    }
    if let Some(p) = s.persons.get_mut(accused) {
        p.jailed_until = Some(day + if guilty { 30 } else { 20 });
        p.remember(day, "arrest", if guilty { format!("{bn} took me for the {} — and had the right of it.", c.kind) } else { format!("{bn} threw me in the Stone Cellar for a {} I never did.", c.kind) }, 95, vec![by.to_string()]);
    }
    if guilty && c.commodity == "coin" {
        let hh = s.persons[accused].household.clone();
        s.transfer_up_to(&hh, &c.victim, "coin", c.amount);
    }
    inc(out, "crime.arrest", true, format!("{bn} has arrested {an} for the {} against {}.", c.kind, s.owner_name(&c.victim)), json!({"crime": crime_id, "accused": accused, "by": by}));
    if !guilty {
        inc(out, "crime.wrongful_arrest", false, format!("[GM] {an} is innocent; the real culprit is {}.", s.person_name(&c.culprit)), json!({"crime": crime_id, "culprit": c.culprit}));
    }
    add_grievance(s, accused, by, &format!("arrested me for the {}", c.kind), if guilty { 20 } else { 45 }, out);
    if !guilty {
        for k in kin_of(s, accused) {
            add_grievance(s, &k, by, &format!("jailed {an} on false witness"), 25, out);
        }
        let accusers: Vec<Id> = s.claims.values().filter(|cl| cl.crime.as_deref() == Some(crime_id) && cl.subject.as_deref() == Some(accused)).filter_map(|cl| cl.origin.clone()).collect();
        for a in accusers {
            add_grievance(s, accused, &a, "bore false witness against me", 35, out);
        }
    }
}

pub fn investigate(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    // releases
    let rel: Vec<Id> = s.persons.values().filter(|p| p.jailed_until.map(|d| d <= day).unwrap_or(false)).map(|p| p.id.clone()).collect();
    for id in rel {
        let p = s.persons.get_mut(&id).unwrap();
        p.jailed_until = None;
        p.remember(day, "release", "Released from the Stone Cellar.".into(), 50, vec![]);
    }
    let open: Vec<Id> = s.crimes.values().filter(|c| c.status == "open").map(|c| c.id.clone()).collect();
    let Some(cap) = watch_captain(s) else { return };
    let wis = (s.persons[&cap].stats.abilities[4] - 10).div_euclid(2);
    for cid in open {
        let mut r = Rng::new(s.meta.seed, "inquiry", day as u64, hash_str(&cid));
        let c = s.crimes[&cid].clone();
        for eid in &c.evidence {
            let e = s.evidence.get_mut(eid).unwrap();
            if e.kind == "footprints" {
                e.quality = (e.quality - 4).max(0);
            }
            if !e.found_by_watch && e.quality > 0 && r.chance((130 + wis * 40) * e.quality / 100) {
                e.found_by_watch = true;
                e.custody.push(format!("day {day}: taken into the Hall's keeping by the watch"));
                if let Some(p) = e.points_to.clone() {
                    let q = e.quality;
                    *s.crimes.get_mut(&cid).unwrap().suspicion.entry(p).or_insert(0) += q;
                }
            }
        }
        let claims: Vec<(Id, Id)> = s.claims.values().filter(|cl| cl.crime.as_deref() == Some(cid.as_str())).filter_map(|cl| cl.subject.clone().map(|sub| (cl.id.clone(), sub))).collect();
        for (clid, subject) in claims {
            let known_elsewhere = s.knowledge.iter().any(|(p, k)| p != &cap && k.contains_key(&clid));
            if known_elsewhere && r.chance(200) && s.learn(&cap, &clid, 70, None) {
                *s.crimes.get_mut(&cid).unwrap().suspicion.entry(subject).or_insert(0) += 38;
            }
        }
        let top = s.crimes[&cid].suspicion.iter().max_by_key(|(k, v)| (**v, (*k).clone())).map(|(k, v)| (k.clone(), *v));
        if let Some((who, score)) = top {
            if score >= 100 && s.persons.get(&who).map(|p| p.alive() && p.jailed_until.is_none()).unwrap_or(false) {
                arrest(s, &cid, &who, &cap, out);
                continue;
            }
        }
        if day - c.day > 45 {
            let cm = s.crimes.get_mut(&cid).unwrap();
            cm.status = "cold".into();
            cm.closed_day = Some(day);
        }
    }
}

pub fn interactions(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let adj = adjacency(s);
    let people: Vec<Person> = s.living().filter(|p| !p.is_pc && p.age(day) >= 12 && p.jailed_until.is_none()).cloned().collect();
    for p in &people {
        let mut r = Rng::new(s.meta.seed, "social", day as u64, hash_str(&p.id));
        if !r.chance(150 + p.traits.sociability * 3) {
            continue;
        }
        let other_id = match adj.get(&p.id) {
            Some(c) if !c.is_empty() && !r.chance(15) => r.pick(c).clone(),
            _ => {
                let pool: Vec<&Person> = people.iter().filter(|q| q.id != p.id && (q.work == p.work || q.home == p.home)).collect();
                if pool.is_empty() {
                    continue;
                }
                r.pick(&pool).id.clone()
            }
        };
        let Some(q) = s.persons.get(&other_id).filter(|q| q.alive()).cloned() else { continue };
        let aff = s.rel(&p.id, &q.id).map(|x| x.affinity()).unwrap_or(40);
        let roll = (p.traits.empathy + q.traits.empathy) / 2 - (p.traits.temper + q.traits.temper) / 4 + r.range(-45, 45) + (aff - 40) / 2;
        for (a, b) in [(&p.id, &q.id), (&q.id, &p.id)] {
            let e = s.rel_mut(a, b);
            e.familiarity = clamp(e.familiarity + 1, 0, 100);
            if roll > 42 {
                e.fondness = clamp(e.fondness + 2, 0, 100);
                e.trust = clamp(e.trust + 1, 0, 100);
            } else if roll < -8 {
                e.fondness = clamp(e.fondness - 3, 0, 100);
                e.trust = clamp(e.trust - 2, 0, 100);
            }
        }
        if roll < -8 && (p.traits.temper > 68 || q.traits.temper > 68) && r.chance(220) {
            let (hot, cool) = if p.traits.temper >= q.traits.temper { (&p.id, &q.id) } else { (&q.id, &p.id) };
            add_grievance(s, cool, hot, "insulted me in front of the street", 7, out);
        }
        update_status(s, &p.id, &q.id, out);
        update_status(s, &q.id, &p.id, out);
    }
}

pub fn rumours(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    if day % 30 == 0 {
        // collective forgetting: old talk fades unless it is lore or tied to an open case
        let stale: Vec<Id> = s.claims.values().filter(|c| day - c.day > 150 && c.kind != "lore" && !c.crime.as_ref().map(|x| s.crimes.get(x).map(|cr| cr.status == "open").unwrap_or(false)).unwrap_or(false)).map(|c| c.id.clone()).collect();
        for id in &stale {
            s.claims.remove(id);
            s.party.knowledge.remove(id);
        }
        for k in s.knowledge.values_mut() {
            k.retain(|c, _| !stale.contains(c));
        }
        s.knowledge.retain(|_, k| !k.is_empty());
        let closed: Vec<Id> = s.crimes.values().filter(|c| c.status != "open" && day - c.day > 150).map(|c| c.id.clone()).collect();
        s.evidence.retain(|_, e| !closed.contains(&e.crime));
    }
    let adj = adjacency(s);
    let recent: Vec<Claim> = s.claims.values().filter(|c| day - c.day <= 40).cloned().collect();
    for c in &recent {
        let knowers: Vec<(Id, i64)> = s.knowledge.iter().filter_map(|(p, k)| k.get(&c.id).map(|kn| (p.clone(), kn.confidence))).collect();
        if c.public {
            continue; // public notices become common knowledge (see WorldState::knows)
        }
        for (k, conf) in knowers {
            let Some(teller) = s.persons.get(&k).filter(|p| p.alive() && !p.is_pc).cloned() else { continue };
            let mut r = Rng::new(s.meta.seed, "rumour", day as u64, hash_str(&k) ^ hash_str(&c.id));
            if conf < 35 || !r.chance(30 + teller.traits.sociability * 2) {
                continue;
            }
            let Some(contacts) = adj.get(&k).filter(|v| !v.is_empty()) else { continue };
            let listener = r.pick(contacts).clone();
            if !s.persons.get(&listener).map(|p| p.alive() && p.age(day) >= 12).unwrap_or(false) || c.subject.as_deref() == Some(listener.as_str()) && r.chance(600) {
                continue;
            }
            let trust = s.rel(&listener, &k).map(|x| x.trust).unwrap_or(40);
            let mut claim_id = c.id.clone();
            // distortion: the story changes in the telling
            if c.subject.is_some() && c.kind != "notice" && r.chance(40) {
                if let Some(pool) = adj.get(&listener).filter(|v| !v.is_empty()) {
                    let wrong = r.pick(pool).clone();
                    if Some(&wrong) != c.subject.as_ref() && s.persons.get(&wrong).map(|p| p.alive()).unwrap_or(false) {
                        let text = format!("Word is that {} was the one seen about that business — or so {} heard.", s.person_name(&wrong), s.person_name(&listener));
                        claim_id = s.add_claim(Claim { kind: "rumour".into(), subject: Some(wrong), crime: c.crime.clone(), text, truth: false, origin: Some(listener.clone()), parent: Some(c.id.clone()), ..Default::default() });
                    }
                }
            }
            let lconf = conf * (50 + trust / 2) / 100;
            if lconf < 25 || !s.learn(&listener, &claim_id, lconf, Some(k.clone())) {
                continue;
            }
            let cl = s.claims[&claim_id].clone();
            if let Some(sub) = &cl.subject {
                if s.relations.contains_key(&rel_key(&listener, sub)) {
                    let e = s.rel_mut(&listener, sub);
                    e.trust = clamp(e.trust - 6, 0, 100);
                    e.respect = clamp(e.respect - 4, 0, 100);
                }
                // victims who hear a name hold it against that person
                if let Some(cr) = cl.crime.as_ref().and_then(|id| s.crimes.get(id)).cloned() {
                    let is_victim = s.persons[&listener].household == cr.victim || s.orgs.get(&cr.victim).map(|o| o.leader.as_ref() == Some(&listener)).unwrap_or(false);
                    if is_victim && lconf >= 45 {
                        add_grievance(s, &listener, sub, &format!("is said to be behind the {}", cr.kind), 20, out);
                    }
                }
            }
        }
    }
}

fn org_leader_of(s: &WorldState, person: &str) -> Option<Id> {
    s.orgs.values().find(|o| o.leader.as_deref() == Some(person)).map(|o| o.id.clone())
}

/// Enemies and nemeses act on their grudges. Every scheme is a real state change
/// with evidence and possible witnesses, so it can be investigated and can backfire.
pub fn schemes(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let edges: Vec<(Id, Id, String)> = s.relations.iter().filter(|(_, r)| (r.status == "enemy" || r.status == "nemesis") && day - r.last_scheme_day >= if r.status == "nemesis" { 7 } else { 21 }).map(|(k, r)| {
        let mut it = k.split('|');
        (it.next().unwrap().to_string(), it.next().unwrap().to_string(), r.status.clone())
    }).collect();
    for (from, to, status) in edges {
        let (Some(a), Some(b)) = (s.persons.get(&from).cloned(), s.persons.get(&to).cloned()) else { continue };
        if !a.alive() || !b.alive() || a.jailed_until.is_some() || a.is_pc || a.age(day) < 14 {
            continue;
        }
        let mut r = Rng::new(s.meta.seed, "scheme", day as u64, hash_str(&from) ^ hash_str(&to).rotate_left(7));
        if !r.chance(if status == "nemesis" { 700 } else { 350 }) {
            continue;
        }
        s.rel_mut(&from, &to).last_scheme_day = day;
        let (an, bn) = (a.full_name(), b.full_name());
        let newest = s.rel(&from, &to).and_then(|x| x.grievances.iter().map(|g| g.day).max()).unwrap_or(day);
        if a.traits.empathy > 65 && day - newest > 60 && r.chance(500) {
            let e = s.rel_mut(&from, &to);
            for g in e.grievances.iter_mut() {
                g.weight /= 3;
            }
            e.fondness = clamp(e.fondness + 10, 0, 100);
            update_status(s, &from, &to, out);
            inc(out, "relation.reconciled", true, format!("{an} has made peace with {bn}."), json!({"from": from, "to": to}));
            continue;
        }
        if b.is_pc {
            // Player agency: NPCs never resolve harm to a PC off-screen. They prepare, and the party meets it in play.
            let p = s.persons.get_mut(&from).unwrap();
            if !p.flags.iter().any(|f| f == "hunting_party") {
                p.flags.push("hunting_party".into());
            }
            if p.tier < 3 {
                p.tier = 3;
                p.agent_profile = "t3-major".into();
            }
            inc(out, "scheme.hunting", false, format!("[GM] {an} is looking for a chance to settle with {bn}."), json!({"from": from, "to": to}));
            continue;
        }
        let same_org_leader = a.employer.is_some() && a.employer == b.employer && org_leader_of(s, &to).is_some();
        let knows_true = s.knowledge.get(&from).map(|k| k.keys().any(|c| s.claims.get(c).map(|cl| cl.subject.as_deref() == Some(to.as_str()) && cl.crime.as_ref().map(|x| s.crimes[x].status == "open").unwrap_or(false)).unwrap_or(false))).unwrap_or(false);
        let w = [
            ("slander", (100 - a.traits.honesty).max(5)),
            ("denounce", if knows_true { 70 } else { 0 }),
            ("sabotage", a.traits.courage * (100 - a.traits.honesty) / 100),
            ("assault", if status == "nemesis" || a.traits.temper > 80 { (a.traits.temper * a.traits.courage / 100 - 25).max(0) } else { 0 }),
            ("undermine", if same_org_leader && a.traits.ambition > 55 { a.traits.ambition } else { 0 }),
        ];
        let total: i64 = w.iter().map(|x| x.1).sum();
        if total <= 0 {
            continue;
        }
        let mut pick = r.range(0, total - 1);
        let mut choice = "slander";
        for (k, v) in w {
            if pick < v {
                choice = k;
                break;
            }
            pick -= v;
        }
        match choice {
            "slander" | "denounce" => {
                let open = s.crimes.values().filter(|c| c.status == "open").max_by_key(|c| (c.day, c.id.clone())).cloned();
                let (text, crime_id, truth) = match &open {
                    Some(c) => (format!("{an} swears {bn} was behind the {} at {}.", c.kind, s.locations[&c.location].name), Some(c.id.clone()), c.culprit == to),
                    None => (format!("{an} says {bn} gives short measure and worse."), None, false),
                };
                let cl = s.add_claim(Claim { kind: "slander".into(), subject: Some(to.clone()), crime: crime_id, text, truth, origin: Some(from.clone()), ..Default::default() });
                s.learn(&from, &cl, 90, None);
                if choice == "denounce" {
                    if let Some(cap) = watch_captain(s) {
                        s.learn(&cap, &cl, 60, Some(from.clone()));
                    }
                }
                inc(out, "scheme.slander", false, format!("[GM] {an} is spreading word against {bn}."), json!({"from": from, "to": to, "claim": cl}));
                if r.chance(400) {
                    add_grievance(s, &to, &from, "has been blackening my name", 15, out);
                }
            }
            "sabotage" => {
                let org = org_leader_of(s, &to);
                let asset = org.as_ref().and_then(|o| s.assets.values().filter(|x| &x.owner == o && x.active && !x.protected).map(|x| x.id.clone()).next());
                if let (Some(o), Some(aid)) = (org, asset) {
                    let (name, loc) = {
                        let x = s.assets.get_mut(&aid).unwrap();
                        x.condition -= 35;
                        if x.condition <= 0 {
                            x.active = false;
                        }
                        (x.name.clone(), x.location.clone())
                    };
                    record_crime(s, "sabotage", &from, &o, &loc, "", 0, &format!("a grudge against {bn}"), out);
                    if !s.assets[&aid].active {
                        inc(out, "asset.destroyed", true, format!("{name} has been wrecked beyond use."), json!({"asset": aid}));
                    }
                } else {
                    let hh = b.household.clone();
                    let g = s.acct(&hh, "grain") * 3 / 10;
                    s.spoil(&hh, "grain", g);
                    record_crime(s, "sabotage", &from, &hh, &b.home, "grain", g, &format!("a grudge against {bn}"), out);
                }
            }
            "assault" => {
                let dmg = r.range(15, 40);
                let loc = b.work.clone();
                let dead = {
                    let t = s.persons.get_mut(&to).unwrap();
                    t.health -= dmg;
                    t.remember(day, "attacked", format!("{an} set on me in the street."), 90, vec![from.clone()]);
                    t.health <= 0
                };
                record_crime(s, if dead { "murder" } else { "assault" }, &from, &to, &loc, "", 0, &format!("a grudge against {bn}"), out);
                if dead {
                    for k in kin_of(s, &to) {
                        add_grievance(s, &k, &from, &format!("killed {bn}"), 60, out);
                    }
                    kill(s, &to, &format!("killed by {an}"), out);
                } else {
                    add_grievance(s, &to, &from, "beat me in the street", 30, out);
                }
            }
            "undermine" => {
                let oid = org_leader_of(s, &to).unwrap();
                let members = s.orgs[&oid].members.clone();
                let standing = |s: &WorldState, x: &str| -> i64 { members.iter().filter(|m| m.as_str() != x).map(|m| s.rel(m, x).map(|e| e.respect + e.fondness / 2).unwrap_or(55)).sum::<i64>() / members.len().max(1) as i64 };
                let (sa, sb) = (standing(s, &from) + a.traits.ambition / 2, standing(s, &to) + b.traits.ambition / 3 + 12);
                if sa + r.range(-15, 15) > sb {
                    s.orgs.get_mut(&oid).unwrap().leader = Some(from.clone());
                    for (id, add) in [(&to, false), (&from, true)] {
                        let p = s.persons.get_mut(id).unwrap();
                        p.flags.retain(|f| f != &format!("leader:{oid}"));
                        if add {
                            p.flags.push(format!("leader:{oid}"));
                            p.tier = p.tier.max(3);
                            if p.agent_profile.starts_with("t1") || p.agent_profile.starts_with("t2") {
                                p.agent_profile = "t3-major".into();
                            }
                        }
                    }
                    inc(out, "org.leader_changed", true, format!("{an} has ousted {bn} as head of {}.", s.orgs[&oid].name), json!({"org": oid, "new": from, "old": to}));
                    add_grievance(s, &to, &from, "took my seat from under me", 45, out);
                } else {
                    inc(out, "org.challenge_failed", true, format!("{an} moved against {bn} for the leadership of {} and failed.", s.orgs[&oid].name), json!({"org": oid, "challenger": from}));
                    add_grievance(s, &to, &from, "tried to unseat me", 25, out);
                }
            }
            _ => {}
        }
    }
}

fn set_policy(s: &mut WorldState, org: &str, key: &str, v: i64) -> bool {
    let o = s.orgs.get_mut(org).unwrap();
    let old = o.policy.insert(key.to_string(), v);
    old != Some(v)
}

fn shift_org(s: &mut WorldState, a: &str, b: &str, delta: i64, why: &str, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let key = org_key(a, b);
    let r = s.org_relations.entry(key).or_default();
    r.score = clamp(r.score + delta, -100, 100);
    let new = if r.score >= 50 { "allied" } else if r.score >= 20 { "friendly" } else if r.score <= -50 { "hostile" } else if r.score <= -20 { "tense" } else { "neutral" };
    if r.status != new {
        let old = std::mem::replace(&mut r.status, new.to_string());
        r.history.push((day, format!("{old} → {new}: {why}")));
        if r.history.len() > 12 {
            r.history.remove(0);
        }
        let (an, bn) = (s.orgs[a].name.clone(), s.orgs[b].name.clone());
        let text = match new {
            "allied" => format!("{an} and {bn} have sworn common cause ({why})."),
            "hostile" => format!("{an} and {bn} are now openly hostile ({why})."),
            "tense" => format!("Relations between {an} and {bn} have soured ({why})."),
            "friendly" => format!("{an} and {bn} are on good terms ({why})."),
            _ => format!("{an} and {bn} have settled into wary neutrality."),
        };
        inc(out, "org.relation", true, text, json!({"a": a, "b": b, "status": new, "score": s.org_relations[&org_key(a, b)].score}));
        if new == "hostile" {
            if let (Some(la), Some(lb)) = (s.orgs[a].leader.clone(), s.orgs[b].leader.clone()) {
                add_grievance(s, &la, &lb, &format!("leads {bn} against us"), 14, out);
                add_grievance(s, &lb, &la, &format!("leads {an} against us"), 14, out);
            }
        }
    }
}

/// Weekly: each organisation pursues its interests with the levers it has.
pub fn org_politics(s: &mut WorldState, out: &mut Vec<Incident>) {
    let hunger = avg_hunger(s);
    let ratio = s.grain_price_ratio_pct();
    let leader_trait = |s: &WorldState, org: &str, f: fn(&Traits) -> i64| s.orgs[org].leader.as_ref().and_then(|l| s.persons.get(l)).map(|p| f(&p.traits)).unwrap_or(50);

    // The Granary Compact: hoard into a rising market if its leader is ambitious enough.
    let hoarding = s.orgs["org_compact"].policy["hoarding"] > 0;
    let edict = s.orgs["org_reeve"].policy["edict_antihoarding"] > 0;
    let want_hoard = ratio > 130 && leader_trait(s, "org_compact", |t| t.ambition - t.empathy / 2) > 25 && !edict;
    if want_hoard && !hoarding {
        set_policy(s, "org_compact", "hoarding", 1);
        inc(out, "org.policy", false, "[GM] The Granary Compact has quietly stopped selling grain and is buying up what reaches the market.".into(), json!({"org": "org_compact", "policy": "hoarding", "value": 1}));
        // the porters who shift the sacks know
        let porters: Vec<Id> = s.orgs["org_compact"].members.iter().filter(|m| s.persons.get(*m).map(|p| p.occupation == "porter").unwrap_or(false)).cloned().collect();
        let c = s.add_claim(Claim { kind: "rumour".into(), subject: s.orgs["org_compact"].leader.clone(), text: "The Compact's granary is full and its doors are shut: they are holding grain back to push the price.".into(), truth: true, ..Default::default() });
        for p in porters {
            s.learn(&p, &c, 90, None);
        }
    } else if hoarding && (ratio < 112 || edict) {
        set_policy(s, "org_compact", "hoarding", 0);
        inc(out, "org.policy", false, "[GM] The Granary Compact has resumed selling grain.".into(), json!({"org": "org_compact", "policy": "hoarding", "value": 0}));
    }
    let hoarding = s.orgs["org_compact"].policy["hoarding"] > 0;
    if hoarding && hunger > 20 {
        // public anger at the factors
        let day = s.meta.day;
        if let Some(l) = s.orgs["org_compact"].leader.clone() {
            let heads: Vec<(Id, Id)> = s.households.values().filter(|h| !h.dissolved && day - h.last_gouge_grievance >= 28).map(|h| (h.id.clone(), h.head.clone())).collect();
            for (hid, head) in heads {
                let hungry = s.persons.get(&head).map(|p| p.alive() && p.hunger > 30).unwrap_or(false);
                let knows = s.knowledge.get(&head).map(|k| k.keys().any(|c| s.claims[c].text.contains("holding grain back"))).unwrap_or(false);
                if hungry && knows {
                    s.households.get_mut(&hid).unwrap().last_gouge_grievance = day;
                    add_grievance(s, &head, &l, "starves the town to fatten the Compact", 12, out);
                }
            }
        }
        shift_org(s, "org_compact", "org_reeve", -8, "grain held back while the town goes hungry", out);
    }

    // The Reeve's Hall: relief, edicts, taxes.
    if set_policy(s, "org_reeve", "relief", (hunger > 28) as i64) && hunger > 28 {
        inc(out, "org.policy", true, "The Reeve has opened the town reserve to the hungry.".into(), json!({"org": "org_reeve", "policy": "relief", "value": 1}));
    }
    if !edict && hunger > 36 && s.acct("org_compact", "grain") > 1500 * M {
        set_policy(s, "org_reeve", "edict_antihoarding", 1);
        inc(out, "org.edict", true, "By the Reeve's seal: all grain held in store must be offered for sale. The Compact's factors are furious.".into(), json!({"org": "org_reeve", "edict": "antihoarding"}));
        shift_org(s, "org_compact", "org_reeve", -22, "the anti-hoarding edict", out);
        shift_org(s, "org_lodge", "org_reeve", 6, "the Lodge backs the edict", out);
    } else if edict && hunger < 15 {
        set_policy(s, "org_reeve", "edict_antihoarding", 0);
        inc(out, "org.edict", true, "The Reeve has lifted the grain edict.".into(), json!({"org": "org_reeve", "edict": "lifted"}));
        shift_org(s, "org_compact", "org_reeve", 6, "the edict is lifted", out);
    }
    let coin = s.acct("org_reeve", "coin");
    let tax = s.orgs["org_reeve"].policy["tax_permille"];
    let new_tax = if coin < 2000 * M { (tax + 10).min(150) } else if coin > 9000 * M { (tax - 10).max(30) } else { tax };
    if set_policy(s, "org_reeve", "tax_permille", new_tax) {
        inc(out, "org.policy", true, format!("Market dues are now {}.{}%.", new_tax / 10, new_tax % 10), json!({"org": "org_reeve", "policy": "tax_permille", "value": new_tax}));
        if new_tax > tax {
            shift_org(s, "org_compact", "org_reeve", -4, "higher market dues", out);
        }
    }
    if coin > 8000 * M {
        shift_org(s, "org_lodge", "org_reeve", 3, "the Hall pays for quay repairs", out);
    }
    if s.org_relations[&org_key("org_compact", "org_reeve")].status == "hostile" {
        let fine = s.acct("org_compact", "coin") * 3 / 100;
        let _ = s.transfer("org_compact", "org_reeve", "coin", fine);
        inc(out, "org.audit", true, format!("The Hall's clerks audited the Compact's books and levied a fine of {} marks.", fine / M), json!({"fine_milli": fine}));
    }

    // The Ferrymen's Lodge: fees and strikes.
    let lcoin = s.acct("org_lodge", "coin");
    let fee = s.orgs["org_lodge"].policy["ferry_fee_milli"];
    let new_fee = if lcoin < 1500 * M { (fee + 25).min(400) } else if lcoin > 6000 * M { (fee - 25).max(75) } else { fee };
    if set_policy(s, "org_lodge", "ferry_fee_milli", new_fee) {
        inc(out, "org.policy", true, format!("The Lodge has set freight fees at {} per thousand measures.", new_fee), json!({"org": "org_lodge", "policy": "ferry_fee_milli", "value": new_fee}));
        shift_org(s, "org_compact", "org_lodge", if new_fee > fee { -5 } else { 3 }, if new_fee > fee { "freight fees raised" } else { "freight fees eased" }, out);
    }
    let striking = s.orgs["org_lodge"].policy["strike"] > 0;
    if !striking && s.orgs["org_lodge"].unpaid_days >= 5 {
        set_policy(s, "org_lodge", "strike", 1);
        inc(out, "org.strike", true, "The ferrymen have shipped their oars: no pay, no crossing.".into(), json!({"org": "org_lodge"}));
        shift_org(s, "org_compact", "org_lodge", -15, "the ferrymen's strike", out);
    } else if striking {
        set_policy(s, "org_lodge", "strike", 0);
        inc(out, "org.strike_ended", true, "The ferrymen are back at the chains.".into(), json!({"org": "org_lodge"}));
    }
    for oid in ["org_compact", "org_lodge", "org_reeve"] {
        if s.orgs[oid].unpaid_days >= 3 {
            if let Some(l) = s.orgs[oid].leader.clone() {
                for m in s.orgs[oid].members.clone() {
                    if m != l {
                        add_grievance(s, &m, &l, "has not paid our wages", 9, out);
                    }
                }
            }
        }
    }
    // leaders' personal regard colours institutional dealings; everything drifts toward neutral
    let pairs: Vec<String> = s.org_relations.keys().cloned().collect();
    for k in pairs {
        let (a, b) = k.split_once('|').unwrap();
        let (a, b) = (a.to_string(), b.to_string());
        let personal = match (s.orgs[&a].leader.clone(), s.orgs[&b].leader.clone()) {
            (Some(la), Some(lb)) => (s.rel(&la, &lb).map(|r| r.affinity() - r.hostility(s.meta.day)).unwrap_or(40) - 40) / 12,
            _ => 0,
        };
        let score = s.org_relations[&k].score;
        let drift = if score > 0 { -1 } else if score < 0 { 1 } else { 0 };
        shift_org(s, &a, &b, personal + drift, "the temper of their leaders", out);
    }
}

fn move_person(s: &mut WorldState, pid: &str, to_hh: &str) {
    let from = s.persons[pid].household.clone();
    if from == to_hh {
        return;
    }
    if let Some(h) = s.households.get_mut(&from) {
        h.members.retain(|m| m != pid);
    }
    let home = s.households[to_hh].home.clone();
    s.households.get_mut(to_hh).unwrap().members.push(pid.to_string());
    let p = s.persons.get_mut(pid).unwrap();
    p.household = to_hh.to_string();
    p.home = home.clone();
    if p.work == from || matches!(p.occupation.as_str(), "elder" | "child") {
        p.work = home;
    }
    settle_household(s, &from);
}

/// After a member leaves or dies: re-seat the head, or dissolve and pass on the estate.
fn settle_household(s: &mut WorldState, hid: &str) {
    let day = s.meta.day;
    let Some(h) = s.households.get(hid).cloned() else { return };
    if h.dissolved {
        return;
    }
    if h.members.is_empty() {
        let heir = s.persons.get(&h.head).map(|p| p.children.clone()).unwrap_or_default().into_iter().filter_map(|c| s.persons.get(&c)).find(|c| c.alive() && c.household != hid).map(|c| c.household.clone()).unwrap_or_else(|| "org_reeve".to_string());
        let keys: Vec<(String, i64)> = s.accounts.iter().filter(|(k, v)| k.starts_with(&format!("{hid}|")) && **v > 0).map(|(k, v)| (k.split('|').nth(1).unwrap().to_string(), *v)).collect();
        for (c, v) in keys {
            let _ = s.transfer(hid, &heir, &c, v);
        }
        for a in s.assets.values_mut() {
            if a.owner == hid {
                a.owner = heir.clone();
            }
        }
        s.households.get_mut(hid).unwrap().dissolved = true;
    } else if !h.members.contains(&h.head) {
        let new_head = h.members.iter().max_by_key(|m| (s.persons[*m].age(day), (*m).clone())).cloned().unwrap();
        s.households.get_mut(hid).unwrap().head = new_head;
    }
}

pub fn kill(s: &mut WorldState, pid: &str, cause: &str, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let Some(p) = s.persons.get(pid).cloned() else { return };
    if !p.alive() {
        return;
    }
    {
        let m = s.persons.get_mut(pid).unwrap();
        m.status = "dead".into();
        m.death_day = Some(day);
        m.jailed_until = None;
    }
    let name = p.full_name();
    inc(out, "person.died", true, format!("{name} ({}, {}) has died: {cause}.", p.occupation, p.age(day)), json!({"person": pid, "cause": cause}));
    for k in kin_of(s, pid) {
        s.persons.get_mut(&k).unwrap().remember(day, "bereavement", format!("{name} died ({cause})."), 92, vec![pid.to_string()]);
    }
    if let Some(sp) = &p.spouse {
        if let Some(w) = s.persons.get_mut(sp) {
            w.spouse = None;
            w.flags.push("widowed".into());
        }
    }
    if let Some(h) = s.households.get_mut(&p.household) {
        h.members.retain(|m| m != pid);
    }
    settle_household(s, &p.household);
    let org_ids: Vec<Id> = s.orgs.keys().cloned().collect();
    for oid in org_ids {
        s.orgs.get_mut(&oid).unwrap().members.retain(|m| m != pid);
        if s.orgs[&oid].leader.as_deref() == Some(pid) {
            succession(s, &oid, &name, out);
        }
    }
    if p.flags.iter().any(|f| f == "watch_captain") {
        if let Some(n) = s.persons.values().filter(|q| q.alive() && q.occupation == "guard").max_by_key(|q| (q.stats.abilities[4], q.id.clone())).map(|q| q.id.clone()) {
            let q = s.persons.get_mut(&n).unwrap();
            q.flags.push("watch_captain".into());
            q.tier = q.tier.max(3);
            q.agent_profile = "t3-major".into();
            inc(out, "org.appointment", true, format!("{} is the new watch-captain.", s.person_name(&n)), json!({"person": n}));
        }
    }
}

fn succession(s: &mut WorldState, oid: &str, old_name: &str, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let next = s.orgs[oid].members.iter().filter_map(|m| s.persons.get(m)).filter(|p| p.alive() && p.age(day) >= 18).max_by_key(|p| (p.traits.ambition + p.stats.level * 10, p.id.clone())).map(|p| p.id.clone());
    s.orgs.get_mut(oid).unwrap().leader = next.clone();
    match next {
        Some(n) => {
            let p = s.persons.get_mut(&n).unwrap();
            p.flags.push(format!("leader:{oid}"));
            p.tier = p.tier.max(3);
            if !p.agent_profile.starts_with("custom") {
                p.agent_profile = "t3-major".into();
            }
            inc(out, "org.leader_changed", true, format!("{} now leads {} after {old_name}.", s.person_name(&n), s.orgs[oid].name), json!({"org": oid, "new": n}));
        }
        None => inc(out, "org.leaderless", true, format!("{} has no one left to lead it.", s.orgs[oid].name), json!({"org": oid})),
    }
}

fn related(s: &WorldState, a: &Person, b: &Person) -> bool {
    a.parents.iter().any(|p| b.parents.contains(p)) || a.parents.contains(&b.id) || b.parents.contains(&a.id) || a.parents.iter().any(|p| s.persons.get(p).map(|pp| pp.parents.iter().any(|g| b.parents.iter().any(|q| s.persons.get(q).map(|qq| qq.parents.contains(g)).unwrap_or(false)))).unwrap_or(false))
}

pub fn courtship(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let singles: Vec<Person> = s.living().filter(|p| !p.is_pc && p.spouse.is_none() && (18..46).contains(&p.age(day)) && p.jailed_until.is_none()).cloned().collect();
    let mut taken: Vec<Id> = vec![];
    for a in &singles {
        if taken.contains(&a.id) {
            continue;
        }
        let mut r = Rng::new(s.meta.seed, "court", day as u64, hash_str(&a.id));
        let cand = singles.iter().filter(|b| b.id > a.id && !taken.contains(&b.id) && !related(s, a, b)).filter(|b| {
            let (x, y) = (s.rel(&a.id, &b.id), s.rel(&b.id, &a.id));
            matches!((x, y), (Some(x), Some(y)) if x.fondness >= 58 && y.fondness >= 58 && x.hostility(day) == 0 && y.hostility(day) == 0)
        }).max_by_key(|b| (s.rel(&a.id, &b.id).map(|x| x.fondness).unwrap_or(0), b.id.clone()));
        let Some(b) = cand else { continue };
        if !r.chance(140) {
            continue;
        }
        taken.push(a.id.clone());
        taken.push(b.id.clone());
        s.persons.get_mut(&a.id).unwrap().spouse = Some(b.id.clone());
        s.persons.get_mut(&b.id).unwrap().spouse = Some(a.id.clone());
        crate::worldgen::bond(s, &a.id, &b.id, "spouse", 80, 80);
        let a_head = s.households[&a.household].head == a.id;
        let b_head = s.households[&b.household].head == b.id;
        if a_head {
            move_person(s, &b.id, &a.household);
        } else if b_head {
            move_person(s, &a.id, &b.household);
        } else {
            let hid = s.next_id("h");
            s.households.insert(hid.clone(), Household { id: hid.clone(), name: format!("House {}", a.family), head: a.id.clone(), members: vec![], home: a.home.clone(), dissolved: false, last_gouge_grievance: -1000 });
            for (pid, old) in [(&a.id, &a.household), (&b.id, &b.household)] {
                let dowry = s.acct(old, "coin") * 12 / 100;
                let food = s.acct(old, "grain") * 12 / 100;
                let _ = s.transfer(old, &hid, "coin", dowry);
                let _ = s.transfer(old, &hid, "grain", food);
                move_person(s, pid, &hid);
            }
        }
        for (x, y) in [(&a.id, &b.id), (&b.id, &a.id)] {
            let yn = s.person_name(y);
            s.persons.get_mut(x).unwrap().remember(day, "marriage", format!("I married {yn}."), 90, vec![y.clone()]);
        }
        inc(out, "person.married", true, format!("{} and {} were wed at the Shrine of the Patient Water.", a.full_name(), b.full_name()), json!({"a": a.id, "b": b.id}));
    }
}

pub fn demography(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    let people: Vec<Person> = s.living().filter(|p| !p.is_pc).cloned().collect();
    for p in &people {
        let mut r = Rng::new(s.meta.seed, "life", day as u64, hash_str(&p.id));
        let age = p.age(day);
        // mortality
        let base = match age {
            0..=4 => 90,
            5..=49 => 20,
            50..=64 => 90,
            65..=74 => 320,
            _ => 900,
        };
        if p.health <= 0 {
            kill(s, &p.id, if p.hunger > 60 { "starvation" } else { "wounds and sickness" }, out);
            continue;
        }
        if r.chance_ppm(base * (200 - p.health) / 100) {
            kill(s, &p.id, if age > 64 { "old age" } else { "fever" }, out);
            continue;
        }
        let birthday = (day - p.birth_day).rem_euclid(DAYS_PER_YEAR) == 0;
        if birthday && age == 15 {
            let parent_occ = p.parents.iter().filter_map(|x| s.persons.get(x)).map(|x| x.occupation.clone()).find(|o| matches!(o.as_str(), "farmer" | "fisher" | "weaver" | "smith" | "woodcutter" | "brewer" | "ferryman" | "porter"));
            let occ = parent_occ.unwrap_or_else(|| "labourer".into());
            let employer = crate::worldgen::employer_for(&occ);
            let m = s.persons.get_mut(&p.id).unwrap();
            m.occupation = occ.clone();
            m.work = crate::worldgen::work_for(&occ, &p.home);
            m.employer = employer.clone();
            if let Some(e) = employer {
                s.orgs.get_mut(&e).unwrap().members.push(p.id.clone());
            }
            inc(out, "person.came_of_age", true, format!("{} has come of age and taken up work as a {occ}.", p.full_name()), json!({"person": p.id, "occupation": occ}));
        }
        if birthday && age == 65 && p.occupation != "elder" {
            let m = s.persons.get_mut(&p.id).unwrap();
            m.occupation = "elder".into();
            m.employer = None;
            let was_captain = m.flags.iter().any(|f| f == "watch_captain");
            m.flags.retain(|f| f != "watch_captain");
            let ids: Vec<Id> = s.orgs.keys().cloned().collect();
            for oid in ids {
                s.orgs.get_mut(&oid).unwrap().members.retain(|x| x != &p.id);
                if s.orgs[&oid].leader.as_deref() == Some(p.id.as_str()) {
                    s.persons.get_mut(&p.id).unwrap().flags.retain(|f| !f.starts_with("leader:"));
                    succession(s, &oid, &p.full_name(), out);
                }
            }
            if was_captain {
                if let Some(n) = s.persons.values().filter(|q| q.alive() && q.occupation == "guard").max_by_key(|q| (q.stats.abilities[4], q.id.clone())).map(|q| q.id.clone()) {
                    let q = s.persons.get_mut(&n).unwrap();
                    q.flags.push("watch_captain".into());
                    q.tier = q.tier.max(3);
                    q.agent_profile = "t3-major".into();
                }
            }
        }
        // births
        if p.sex == "f" && (18..43).contains(&age) && p.jailed_until.is_none() {
            let Some(sp) = p.spouse.as_ref().and_then(|x| s.persons.get(x)).filter(|x| x.alive() && x.sex == "m").cloned() else { continue };
            let hungry = p.hunger > 40;
            if !hungry && p.children.len() < 7 && r.chance_ppm(750) {
                let id = s.next_id("p");
                let sex = if r.chance(500) { "m" } else { "f" };
                let names: &[&str] = if sex == "m" { &["Aldren", "Bram", "Corvin", "Dain", "Edrik", "Fenn", "Hale", "Joss", "Lorn", "Nial", "Oswin", "Perrin", "Tobin", "Wend"] } else { &["Ansa", "Brenna", "Dessa", "Elowen", "Fira", "Gwen", "Kerra", "Liss", "Maren", "Nessa", "Orla", "Tamsin", "Una", "Wynne"] };
                let blend = |a: i64, b: i64, r: &mut Rng| clamp((a + b) / 2 + r.bell(0, 25), 5, 95);
                let (ta, tb) = (&p.traits, &sp.traits);
                let traits = Traits { ambition: blend(ta.ambition, tb.ambition, &mut r), empathy: blend(ta.empathy, tb.empathy, &mut r), courage: blend(ta.courage, tb.courage, &mut r), honesty: blend(ta.honesty, tb.honesty, &mut r), sociability: blend(ta.sociability, tb.sociability, &mut r), temper: blend(ta.temper, tb.temper, &mut r), piety: blend(ta.piety, tb.piety, &mut r), curiosity: blend(ta.curiosity, tb.curiosity, &mut r) };
                let family = s.persons.get(&s.households[&p.household].head).map(|h| h.family.clone()).unwrap_or_else(|| p.family.clone());
                let child = Person { id: id.clone(), name: r.pick(names).to_string(), family, sex: sex.into(), birth_day: day, status: "alive".into(), household: p.household.clone(), home: p.home.clone(), work: p.home.clone(), occupation: "child".into(), tier: 1, traits, health: 80, hunger: 5, parents: vec![sp.id.clone(), p.id.clone()], stats: crate::worldgen::stat_block(&mut r, "child"), agent_profile: "t1-resident".into(), languages: vec!["Sabline".into()], ..Default::default() };
                let cname = child.full_name();
                s.persons.insert(id.clone(), child);
                s.households.get_mut(&p.household).unwrap().members.push(id.clone());
                for par in [&p.id, &sp.id] {
                    let sibs = s.persons[par].children.clone();
                    s.persons.get_mut(par).unwrap().children.push(id.clone());
                    crate::worldgen::bond(s, &id, par, "kin", 75, 78);
                    for sib in sibs {
                        if s.persons[&sib].alive() {
                            crate::worldgen::bond(s, &id, &sib, "kin", 62, 64);
                        }
                    }
                }
                inc(out, "person.born", true, format!("{cname} was born to {} and {}.", p.full_name(), sp.full_name()), json!({"person": id, "mother": p.id, "father": sp.id}));
            }
        }
    }
    if day % 30 == 0 {
        migration(s, out);
    }
}

fn migration(s: &mut WorldState, out: &mut Vec<Incident>) {
    let day = s.meta.day;
    // occupational mobility: labour follows prices, and the Hall keeps its watch manned
    let trades = [("grain", "farmer"), ("fish", "fisher"), ("ale", "brewer"), ("cloth", "weaver"), ("timber", "woodcutter"), ("tools", "smith")];
    let mut ranked: Vec<(i64, &str, &str)> = trades.iter().map(|(g, o)| (s.market.prices[*g] * 100 / s.market.base[*g], *g, *o)).collect();
    ranked.sort();
    let labourers: Vec<Id> = s.persons.values().filter(|p| can_work(p, day) && p.occupation == "labourer").map(|p| p.id.clone()).collect();
    let mut li = labourers.iter();
    let guards = s.persons.values().filter(|p| can_work(p, day) && p.occupation == "guard").count();
    let mut switches: Vec<(Id, String)> = vec![];
    if guards < 4 {
        if let Some(l) = li.next() {
            switches.push((l.clone(), "guard".into()));
        }
    }
    if let Some((ratio, _, occ)) = ranked.last() {
        if *ratio > 140 {
            for l in li.by_ref().take(2) {
                switches.push((l.clone(), occ.to_string()));
            }
        }
    }
    if let Some((ratio, _, occ)) = ranked.first() {
        if *ratio < 50 {
            if let Some(p) = s.persons.values().find(|p| can_work(p, day) && p.occupation == *occ) {
                switches.push((p.id.clone(), "labourer".into()));
            }
        }
    }
    for (pid, occ) in switches {
        let old_emp = s.persons[&pid].employer.clone();
        if let Some(e) = old_emp {
            s.orgs.get_mut(&e).unwrap().members.retain(|m| m != &pid);
        }
        let emp = crate::worldgen::employer_for(&occ);
        if let Some(e) = &emp {
            s.orgs.get_mut(e).unwrap().members.push(pid.clone());
        }
        let p = s.persons.get_mut(&pid).unwrap();
        let old = std::mem::replace(&mut p.occupation, occ.clone());
        p.employer = emp;
        p.work = crate::worldgen::work_for(&occ, &p.home.clone());
        p.stats = Stats { level: p.stats.level, ..crate::worldgen::stat_block(&mut Rng::new(s.meta.seed, "retrain", day as u64, hash_str(&pid)), &occ) };
        let name = p.full_name();
        inc(out, "person.changed_trade", true, format!("{name} has given up work as a {old} to become a {occ}."), json!({"person": pid, "from": old, "to": occ}));
    }
    let hunger = avg_hunger(s);
    let mut r = Rng::new(s.meta.seed, "migration", day as u64, 0);
    if hunger < 12 && s.population() < 262 && r.chance(300) {
        let fam = format!("{}{}", r.pick(&["Bar", "Cray", "Dell", "Holm", "Kes", "Mor", "Pell", "Rook", "Stave", "Tarn"]), r.pick(&["wick", "ford", "mere", "holt", "by", "croft", "lock"]));
        let hid = s.next_id("h");
        s.households.insert(hid.clone(), Household { id: hid.clone(), name: format!("House {fam}"), head: String::new(), members: vec![], home: "loc_southgate".into(), dissolved: false, last_gouge_grievance: -1000 });
        let mut ids = vec![];
        for (sex, names) in [("m", ["Arlo", "Cade", "Emrys", "Teague"]), ("f", ["Avice", "Bryn", "Edda", "Thea"])] {
            let id = s.next_id("p");
            let occ = r.pick(&["labourer", "weaver", "fisher", "woodcutter", "labourer"]).to_string();
            let age = r.range(20, 36);
            let p = Person { id: id.clone(), name: r.pick(&names).to_string(), family: fam.clone(), sex: sex.into(), birth_day: day - age * DAYS_PER_YEAR - r.range(0, 359), status: "alive".into(), household: hid.clone(), home: "loc_southgate".into(), work: crate::worldgen::work_for(&occ, "loc_southgate"), occupation: occ.clone(), tier: 1, traits: crate::worldgen::random_traits(&mut r), health: 85, hunger: 10, stats: crate::worldgen::stat_block(&mut r, &occ), agent_profile: "t1-resident".into(), languages: vec!["Sabline".into(), "Brackish".into()], flags: vec!["newcomer".into()], ..Default::default() };
            s.persons.insert(id.clone(), p);
            s.households.get_mut(&hid).unwrap().members.push(id.clone());
            ids.push(id);
        }
        s.households.get_mut(&hid).unwrap().head = ids[0].clone();
        s.persons.get_mut(&ids[0]).unwrap().spouse = Some(ids[1].clone());
        s.persons.get_mut(&ids[1]).unwrap().spouse = Some(ids[0].clone());
        crate::worldgen::bond(s, &ids[0], &ids[1], "spouse", 78, 80);
        let _ = s.transfer("abroad", &hid, "coin", 140 * M);
        let _ = s.transfer("abroad", &hid, "grain", 20 * M);
        inc(out, "migration.arrival", true, format!("The {fam} family has come up the river to settle in Southgate Rows."), json!({"household": hid}));
    } else if hunger > 55 {
        let leaders: Vec<Id> = s.orgs.values().filter_map(|o| o.leader.clone()).collect();
        let cand = s.households.values().filter(|h| !h.dissolved && !h.members.is_empty() && !h.members.iter().any(|m| leaders.contains(m) || s.persons[m].is_pc)).max_by_key(|h| (h.members.iter().map(|m| s.persons[m].hunger).sum::<i64>() / h.members.len() as i64, h.id.clone())).cloned();
        if let Some(h) = cand {
            for m in &h.members {
                let p = s.persons.get_mut(m).unwrap();
                p.status = "emigrated".into();
                p.death_day = Some(day);
                let ids: Vec<Id> = s.orgs.keys().cloned().collect();
                for oid in ids {
                    s.orgs.get_mut(&oid).unwrap().members.retain(|x| x != m);
                }
            }
            let coin = s.acct(&h.id, "coin");
            let _ = s.transfer(&h.id, "abroad", "coin", coin);
            s.households.get_mut(&h.id).unwrap().members.clear();
            settle_household(s, &h.id);
            inc(out, "migration.departure", true, format!("{} has left Marrowick to look for bread elsewhere.", h.name), json!({"household": h.id}));
        }
    }
}

/// Importance-based cognition budget (Decision 07). Tier is a compute/priority
/// selector, not an intelligence score. T3/T4 are never demoted automatically.
pub fn refresh_tiers(s: &mut WorldState) {
    let mut hot: BTreeMap<Id, u8> = BTreeMap::new();
    for (k, r) in &s.relations {
        if r.status == "enemy" || r.status == "nemesis" {
            let (a, b) = k.split_once('|').unwrap();
            let to_pc = s.persons.get(b).map(|p| p.is_pc).unwrap_or(false);
            let t = if r.status == "nemesis" && to_pc { 3 } else { 2 };
            let e = hot.entry(a.to_string()).or_insert(0);
            *e = (*e).max(t);
            hot.entry(b.to_string()).or_insert(2);
        }
    }
    for c in s.crimes.values().filter(|c| c.status == "open") {
        hot.entry(c.culprit.clone()).or_insert(2);
    }
    for p in s.persons.values_mut() {
        if !p.alive() || p.is_pc || p.tier >= 3 {
            continue;
        }
        let mut t = 1u8;
        if matches!(p.occupation.as_str(), "factor" | "clerk" | "innkeeper" | "healer" | "priest" | "mage") || p.talks_with_party >= 2 {
            t = 2;
        }
        t = t.max(hot.get(&p.id).copied().unwrap_or(1));
        if p.talks_with_party >= 6 {
            t = 3;
        }
        p.tier = t;
        p.agent_profile = match t {
            3 => "t3-major",
            2 => "t2-recurring",
            _ => "t1-resident",
        }
        .into();
    }
}
