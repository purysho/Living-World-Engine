//! Epistemic separation (Decision 24). This module builds what a given viewer
//! is allowed to know. NPC prompt context is assembled *only* from that NPC's
//! own knowledge projection — objective truth such as a crime's culprit is
//! never fetched and then "hidden by instruction".

use crate::model::*;
use crate::play::{present, slot, whereabouts};
use serde_json::{json, Value};

pub fn trait_words(t: &Traits) -> Vec<&'static str> {
    let mut w = vec![];
    let mut pick = |v: i64, hi: &'static str, lo: &'static str| {
        if v >= 68 {
            w.push(hi)
        } else if v <= 32 {
            w.push(lo)
        }
    };
    pick(t.ambition, "ambitious", "content with their lot");
    pick(t.empathy, "kind-hearted", "hard-hearted");
    pick(t.courage, "bold", "timid");
    pick(t.honesty, "scrupulously honest", "loose with the truth");
    pick(t.sociability, "talkative", "reserved");
    pick(t.temper, "quick-tempered", "even-tempered");
    pick(t.piety, "devout", "irreverent");
    pick(t.curiosity, "nosy", "incurious");
    if w.is_empty() {
        w.push("unremarkable in temperament");
    }
    w
}

fn mood(p: &Person) -> &'static str {
    if p.jailed_until.is_some() {
        "bitter and cold in the Stone Cellar"
    } else if p.hunger > 60 {
        "desperate with hunger"
    } else if p.hunger > 30 {
        "hungry and short-tempered"
    } else if p.health < 40 {
        "ill and weary"
    } else {
        "in ordinary spirits"
    }
}

pub fn time_string(minute: i64) -> String {
    format!("{:02}:{:02} ({})", minute / 60, minute % 60, slot(minute))
}

/// Everything an NPC may draw on when speaking or deciding. No GM-only truth.
pub fn npc_context(s: &WorldState, id: &str) -> Res<Value> {
    let Some(p) = s.persons.get(id) else { return reject("unknown_entity", format!("no person '{id}'")) };
    let day = s.meta.day;
    let lead = s.party.members.first().cloned();
    let hh = s.households.get(&p.household);
    let family: Vec<String> = hh.map(|h| h.members.iter().filter(|m| *m != id).filter_map(|m| s.persons.get(m)).map(|m| format!("{} ({}, {})", m.full_name(), m.occupation, m.age(day))).collect()).unwrap_or_default();
    let org = s.org_of(id).map(|o| json!({"name": o.name, "is_leader": o.leader.as_deref() == Some(id), "charter": o.charter, "goals": o.goals, "unpaid_days": o.unpaid_days}));
    let mut rels: Vec<(i64, Value)> = s.relations.iter().filter(|(k, _)| k.starts_with(&format!("{id}|"))).filter_map(|(k, r)| {
        let other = k.split('|').nth(1)?;
        let o = s.persons.get(other)?;
        let weight = r.hostility(day) * 3 + (r.affinity() - 50).abs() + if r.kinds.iter().any(|x| x == "spouse") { 40 } else { 0 };
        Some((weight, json!({"who": o.full_name(), "their_trade": o.occupation, "alive": o.alive(), "bond": r.kinds, "regard": r.status, "grievances": r.grievances.iter().map(|g| g.what.clone()).collect::<Vec<_>>()})))
    }).collect();
    rels.sort_by_key(|x| std::cmp::Reverse(x.0));
    let mut mems: Vec<(i64, &Memory)> = p.memories.iter().map(|m| (m.salience * 3 - (day - m.day).min(200), m)).collect();
    mems.sort_by_key(|x| std::cmp::Reverse(x.0));
    let mut claims: Vec<(Id, i64)> = s.known_claims(id);
    claims.sort_by_key(|(c, conf)| (std::cmp::Reverse(s.claims.get(c).map(|x| x.day).unwrap_or(0)), std::cmp::Reverse(*conf)));
    let knowledge: Vec<Value> = claims.iter().take(14).filter_map(|(c, conf)| s.claims.get(c).map(|cl| json!({"id": c, "text": cl.text, "confidence": conf, "kind": cl.kind, "day_heard": cl.day}))).collect();
    let toward_party = lead.as_ref().and_then(|l| s.rel(id, l)).map(|r| json!({"regard": r.status, "trust": r.trust, "fondness": r.fondness, "grievances": r.grievances.iter().map(|g| g.what.clone()).collect::<Vec<_>>()}));
    let conv: Vec<Value> = s.party.conversations.get(id).map(|c| c.iter().map(|l| json!({"who": l.who, "text": l.text})).collect()).unwrap_or_default();
    let hh_food = s.acct(&p.household, "grain") + s.acct(&p.household, "fish");
    Ok(json!({
        "you": {"name": p.full_name(), "age": p.age(day), "sex": p.sex, "trade": p.occupation, "appearance": p.appearance, "temperament": trait_words(&p.traits), "mood": mood(p), "languages": p.languages, "notable": p.flags.iter().filter(|f| !f.starts_with("hunting")).collect::<Vec<_>>()},
        "household": {"name": hh.map(|h| h.name.clone()), "members": family, "coin_marks": s.acct(&p.household, "coin") / M, "days_of_food": hh.map(|h| hh_food / h.members.len().max(1) as i64 / 900).unwrap_or(0)},
        "organisation": org,
        "goals": p.goals,
        "relationships": rels.into_iter().take(8).map(|x| x.1).collect::<Vec<_>>(),
        "memories": mems.into_iter().take(8).map(|(_, m)| json!({"days_ago": day - m.day, "text": m.text})).collect::<Vec<_>>(),
        "your_knowledge": knowledge,
        "feeling_toward_speaker": toward_party,
        "conversation_so_far": conv,
        "here_and_now": {"place": s.locations.get(&whereabouts(s, p)).map(|l| l.name.clone()), "date": date_string(day), "time": time_string(s.party.minute), "weather": s.weather.desc, "grain_price_vs_normal_pct": s.grain_price_ratio_pct()},
    }))
}

/// Deterministic, offline dialogue: built strictly from the same context a model would get.
pub fn offline_reply(s: &WorldState, id: &str, said: &str) -> Res<(String, Vec<Id>, i64)> {
    let Some(p) = s.persons.get(id) else { return reject("unknown_entity", "no such person") };
    let day = s.meta.day;
    let lead = s.party.members.first().cloned().unwrap_or_default();
    let rel = s.rel(id, &lead);
    let regard = rel.map(|r| r.status.as_str()).unwrap_or("neutral");
    let trust = rel.map(|r| r.trust).unwrap_or(40);
    let low = said.to_lowercase();
    let mut out = String::new();
    let mut disclosed = vec![];
    let mut attitude = 0;
    if matches!(regard, "enemy" | "nemesis") {
        let why = rel.and_then(|r| r.grievances.last()).map(|g| g.what.clone()).unwrap_or_else(|| "what you did".into());
        return Ok((format!("\"You.\" {} does not blink. \"You {}. I have nothing to say to you that I'd say in daylight.\"", p.name, why.replace(" me ", " me, ")), vec![], -1));
    }
    let rude = ["idiot", "fool", "peasant", "shut up", "or else", "threat"].iter().any(|w| low.contains(w));
    let kind = ["please", "thank", "sorry", "help you", "kind"].iter().any(|w| low.contains(w));
    if rude {
        attitude = -2;
        out += if p.traits.temper > 60 { "\"Mind your tongue in my town.\" " } else { "\"There's no call for that.\" " };
    } else if kind {
        attitude = 1;
    }
    // opening, coloured by mood and trade
    let opener = if p.hunger > 45 {
        format!("\"Forgive me, I've not eaten properly in days. Grain's at {}% of what it should be.\" ", s.grain_price_ratio_pct())
    } else if low.contains("who are you") || low.contains("your name") || low.contains("hello") || low.contains("greet") || low.len() < 12 {
        format!("\"{}. {} by trade, {} all my life.\" ", p.full_name(), cap(&p.occupation), s.meta.town)
    } else {
        String::new()
    };
    out += &opener;
    // topic matching against what this person actually knows
    let known = s.known_claims(id);
    let asks_crime = ["theft", "stole", "stolen", "thief", "crime", "murder", "kill", "attack", "saw", "seen", "witness", "who did", "missing", "rumour", "rumor", "heard", "news", "gossip", "happen"].iter().any(|w| low.contains(w));
    let mut scored: Vec<(i64, &Claim, i64)> = known.iter().filter_map(|(c, conf)| s.claims.get(c).map(|cl| (cl, *conf))).map(|(cl, conf)| {
        let mut sc = 0;
        for word in low.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 3) {
            if cl.text.to_lowercase().contains(word) {
                sc += 6;
            }
        }
        if asks_crime && cl.crime.is_some() {
            sc += 5;
        }
        if asks_crime && cl.kind != "lore" {
            sc += 2;
        }
        sc += (cl.day - day + 30).max(0) / 10;
        (sc, cl, conf)
    }).filter(|x| x.0 >= 5).collect();
    scored.sort_by_key(|x| (std::cmp::Reverse(x.0), x.1.id.clone()));
    let guarded = trust < 30 && p.traits.sociability < 50;
    let implicates_self_or_kin = |cl: &Claim| cl.subject.as_deref().map(|sub| sub == id || p.spouse.as_deref() == Some(sub) || p.children.iter().any(|c| c == sub) || p.parents.iter().any(|c| c == sub)).unwrap_or(false);
    let mut told = 0;
    for (_, cl, conf) in scored.iter().take(3) {
        if implicates_self_or_kin(cl) {
            out += "\"I'd not know anything about that.\" They look away a little too quickly. ";
            continue;
        }
        if guarded && cl.kind != "notice" && cl.kind != "lore" {
            out += "\"I keep my own counsel with strangers.\" ";
            break;
        }
        let hedge = if *conf >= 80 { "I'll tell you what I know for certain: " } else if *conf >= 50 { "What I heard is this: " } else { "It's only talk, mind, but: " };
        out += &format!("\"{hedge}{}\" ", cl.text);
        disclosed.push(cl.id.clone());
        told += 1;
        if told >= 2 {
            break;
        }
    }
    if told == 0 && !out.contains("counsel") {
        if low.contains("price") || low.contains("grain") || low.contains("market") || low.contains("trade") {
            let r = s.grain_price_ratio_pct();
            out += &format!("\"Grain's {} at the Saltmarket — {} a measure. {}\" ", if r > 140 { "dear" } else if r < 80 { "cheap" } else { "fair" }, fmt_price(s.market.prices["grain"]), if r > 140 { "Folk say the Compact's doors are shut for a reason." } else { "No complaints this season." });
        } else if low.contains("reeve") || low.contains("compact") || low.contains("lodge") || low.contains("guild") || low.contains("hall") {
            let o = s.org_of(id);
            out += &match o {
                Some(o) => format!("\"I'm sworn to {}. You'll not hear me speak against it — not to you.\" ", o.name),
                None => "\"The Hall taxes, the Compact weighs, the Lodge ferries, and the rest of us pay all three.\" ".to_string(),
            };
        } else if low.contains("weir") || low.contains("magic") || low.contains("strange") || low.contains("dream") {
            out += "\"The Old Weir? Fish won't swim that pool. I don't go up there on a tally night and nor should you.\" ";
        } else if asks_crime {
            out += "\"I've heard nothing worth repeating.\" ";
        } else if let Some(m) = p.memories.iter().filter(|m| m.kind != "crime").max_by_key(|m| m.salience * 3 - (day - m.day)) {
            out += &format!("\"What's on my mind? {}\" ", m.text.trim_end_matches('.'));
        } else {
            out += &format!("\"{} — work to do, same as every day. {}\" ", cap(&s.weather.desc.split(';').next().unwrap_or("fair").to_string()), if p.traits.sociability > 60 { "Come by the Drowned Lantern of an evening if you want talk." } else { "Good day to you." });
        }
    }
    Ok((out.trim().to_string(), disclosed, attitude))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}
pub fn fmt_price(milli: i64) -> String {
    format!("{}.{:02} marks", milli / 1000, milli % 1000 / 10)
}

/// Presentation-only content filter (Decision 29, AT-018). Canon is untouched.
pub fn soften(s: &WorldState, kind: &str, text: &str) -> String {
    let level = s.campaign.as_ref().and_then(|c| c.content.get("violence")).map(|x| x.as_str()).unwrap_or("allowed");
    let violent = ["murder", "assault", "combat", "died", "killed", "attacked"].iter().any(|k| kind.contains(k) || text.to_lowercase().contains(k));
    if violent && (level == "veil" || level == "excluded") {
        "Something violent happened; the details stay offscreen for this table.".to_string()
    } else if violent && level == "limited" {
        text.replace("killed", "slain").replace("beat me down", "bested me").replace("is dead", "has fallen")
    } else {
        text.to_string()
    }
}

/// What the party can perceive right now.
pub fn scene(s: &WorldState) -> Res<Value> {
    let Some(loc) = s.locations.get(&s.party.location) else { return reject("no_party", "party has no location") };
    let lead = s.party.members.first().cloned().unwrap_or_default();
    let day = s.meta.day;
    let people: Vec<Value> = present(s, &loc.id).iter().take(40).map(|id| {
        let p = &s.persons[id];
        let r = s.rel(id, &lead);
        json!({"id": id, "name": if p.talks_with_party > 0 || p.tier >= 3 { p.full_name() } else { format!("a {} ({})", p.occupation, p.appearance) }, "known": p.talks_with_party > 0 || p.tier >= 3, "trade": p.occupation, "appearance": p.appearance, "age": p.age(day), "regard": r.map(|x| x.status.clone()).unwrap_or("neutral".into()), "hostile": p.flags.iter().any(|f| f == "hunting_party"), "jailed": p.jailed_until.is_some(), "tier": p.tier})
    }).collect();
    let exits: Vec<Value> = loc.links.iter().map(|(l, m)| json!({"id": l, "name": s.locations[l].name, "minutes": m})).collect();
    let notices: Vec<Value> = s.claims.values().filter(|c| c.public && day - c.day <= 14).map(|c| json!({"id": c.id, "text": soften(s, &c.kind, &c.text), "day": c.day})).collect();
    let pcs: Vec<Value> = s.party.members.iter().filter_map(|m| s.persons.get(m)).map(|p| json!({"id": p.id, "name": p.full_name(), "class": p.stats.class, "level": p.stats.level, "hp": p.stats.hp, "max_hp": p.stats.max_hp, "ac": p.stats.ac, "abilities": p.stats.abilities, "exhaustion": p.stats.exhaustion, "potions": p.stats.potions, "jailed": p.jailed_until.is_some()})).collect();
    let stores: Vec<Value> = s.orgs.values().filter(|o| o.seat == loc.id).map(|o| json!({"id": o.id, "name": o.name})).collect();
    Ok(json!({
        "location": {"id": loc.id, "name": loc.name, "kind": loc.kind, "desc": loc.desc},
        "date": date_string(day), "time": time_string(s.party.minute), "season": season(day), "weather": s.weather.desc,
        "people": people, "exits": exits, "notices": notices, "party": pcs, "stores": stores,
        "purse": {"coin": s.acct("party", "coin"), "grain": s.acct("party", "grain"), "fish": s.acct("party", "fish"), "ale": s.acct("party", "ale"), "cloth": s.acct("party", "cloth"), "timber": s.acct("party", "timber"), "tools": s.acct("party", "tools")},
        "prices": if loc.id == "loc_market" || loc.id == "loc_granary" { json!(s.market.prices) } else { Value::Null },
        "encounter": s.encounter, "rules": crate::rules::profile(s), "campaign": s.campaign,
        "ambient_potential": s.magic.ambient / M,
    }))
}

pub fn narrate_offline(s: &WorldState) -> Res<String> {
    let sc = scene(s)?;
    let n = sc["people"].as_array().map(|a| a.len()).unwrap_or(0);
    let hungry = crate::sim::avg_hunger(s);
    let mut t = format!("{} — {}, {}. {} {}", sc["location"]["name"].as_str().unwrap_or(""), sc["date"].as_str().unwrap_or(""), sc["time"].as_str().unwrap_or(""), sc["location"]["desc"].as_str().unwrap_or(""), cap(s.weather.desc.as_str()));
    t += &match n {
        0 => ". Nobody is about.".to_string(),
        1..=4 => format!(". A few people are here ({n})."),
        _ => format!(". The place is busy: {n} people about their business."),
    };
    if hungry > 30 {
        t += " Faces are thin, and conversations stop when a stranger comes near.";
    }
    if sc["people"].as_array().map(|a| a.iter().any(|p| p["hostile"].as_bool().unwrap_or(false))).unwrap_or(false) {
        t += " Someone here is watching you with open hatred.";
    }
    Ok(t)
}

/// Reduce a person to what any resident could observe (player view).
pub fn person_public(s: &WorldState, p: &Person) -> Value {
    let day = s.meta.day;
    json!({"id": p.id, "name": p.full_name(), "sex": p.sex, "age": p.age(day), "status": p.status, "occupation": p.occupation, "household": p.household, "home": p.home, "tier": p.tier, "appearance": p.appearance, "employer": p.employer, "jailed": p.jailed_until.is_some(), "is_pc": p.is_pc,
        "flags": p.flags.iter().filter(|f| f.starts_with("leader:") || *f == "watch_captain" || *f == "widowed" || *f == "newcomer").collect::<Vec<_>>()})
}
