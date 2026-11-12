//! The single transition function. Every canonical state change is
//! `apply(state, event_type, payload)`; it either fully succeeds or returns a
//! `Reject` and the caller discards the candidate state (atomicity).
//! No UI, model or mod has any other way to change the world (SIM-007).

use crate::model::*;
use crate::rules;
use crate::sim::social::{add_grievance, kill};
use crate::sim::{inc, step_day, Incident};
use serde_json::{json, Value};

pub const EVENT_VERSION: i64 = 1;

fn gs<'a>(p: &'a Value, k: &str) -> Res<&'a str> {
    p.get(k).and_then(|v| v.as_str()).ok_or(Reject { code: "invalid_payload".into(), message: format!("missing string field '{k}'") })
}
fn gi(p: &Value, k: &str) -> Res<i64> {
    p.get(k).and_then(|v| v.as_i64()).ok_or(Reject { code: "invalid_payload".into(), message: format!("missing integer field '{k}'") })
}

/// Roles allowed to originate each event type (default deny).
pub fn allowed_roles(event_type: &str) -> &'static [&'static str] {
    match event_type {
        "world.genesis" | "magic.source_opened" | "fact.protected" => &["world_owner"],
        "sim.tick" | "campaign.configured" | "weather.shock" | "asset.destroyed" | "asset.sold" | "resource.issued" | "resource.transferred" | "fact.recorded" | "npc.edited" | "org.policy_set" | "relation.grievance" | "person.killed" | "merge.marker" => &["world_owner", "gm"],
        "npc.action" => &["world_owner", "gm", "agent"],
        "pc.created" => &["world_owner", "gm", "player"],
        t if t.starts_with("play.") || t.starts_with("combat.") => &["world_owner", "gm", "player"],
        _ => &[],
    }
}

/// Event types a merge may carry from one branch into another.
pub fn mergeable(event_type: &str) -> bool {
    matches!(event_type, "weather.shock" | "asset.destroyed" | "asset.sold" | "resource.issued" | "resource.transferred" | "fact.recorded" | "fact.protected" | "npc.edited" | "org.policy_set" | "relation.grievance" | "person.killed" | "magic.source_opened")
}

pub fn privacy_of(event_type: &str, payload: &Value) -> &'static str {
    match event_type {
        "npc.edited" | "relation.grievance" | "npc.action" | "weather.shock" | "org.policy_set" => "gm_private",
        "fact.recorded" if payload.get("gm_only").and_then(|v| v.as_bool()).unwrap_or(false) => "gm_private",
        _ => "public",
    }
}

pub fn summarize(s: &WorldState, event_type: &str, p: &Value) -> String {
    let name = |k: &str| p.get(k).and_then(|v| v.as_str()).map(|id| s.owner_name(id)).unwrap_or_default();
    match event_type {
        "world.genesis" => format!("The world '{}' begins.", p["name"].as_str().unwrap_or("")),
        "sim.tick" => format!("{} — {}.", date_string(s.meta.day), s.weather.desc),
        "campaign.configured" => format!("Campaign '{}' configured ({} rules, {} GM).", p["name"].as_str().unwrap_or(""), p["rules"].as_str().unwrap_or(""), p["gm_mode"].as_str().unwrap_or("")),
        "pc.created" => format!("{} the {} arrives in Marrowick.", p["name"].as_str().unwrap_or(""), p["class"].as_str().unwrap_or("")),
        "weather.shock" => format!("[GM] Weather shock: {} for {} days.", p["kind"].as_str().unwrap_or(""), p["days"].as_i64().unwrap_or(0)),
        "asset.destroyed" => format!("{} was destroyed ({}).", name("asset_id"), p["reason"].as_str().unwrap_or("cause unknown")),
        "asset.sold" => format!("{} was sold to {} for {} marks.", name("asset_id"), name("buyer"), p["price_milli"].as_i64().unwrap_or(0) / M),
        "resource.issued" => format!("{} {} issued to {} ({}).", p["amount_milli"].as_i64().unwrap_or(0) / M, p["commodity"].as_str().unwrap_or(""), name("owner"), p["reason"].as_str().unwrap_or("")),
        "resource.transferred" => format!("{} {} moved from {} to {}.", p["amount_milli"].as_i64().unwrap_or(0) / M, p["commodity"].as_str().unwrap_or(""), name("from"), name("to")),
        "fact.recorded" => format!("Recorded: {}", p["text"].as_str().unwrap_or("")),
        "fact.protected" => format!("Fact {} is now protected canon.", p["fact_id"].as_str().unwrap_or("")),
        "person.killed" => format!("{} died ({}).", name("person"), p["cause"].as_str().unwrap_or("")),
        "play.moved" => format!("The party went to {}.", p["to"].as_str().and_then(|l| s.locations.get(l)).map(|l| l.name.clone()).unwrap_or_default()),
        "play.talked" => format!("The party spoke with {}.", name("npc")),
        "play.waited" => format!("The party waited {} minutes.", p["minutes"].as_i64().unwrap_or(0)),
        "play.rested" => "The party took a long rest.".into(),
        "play.searched" => "The party searched the area.".into(),
        "play.accused" => format!("The party accused {} before the watch.", name("person")),
        "play.traded" => format!("The party traded {}.", p["good"].as_str().unwrap_or("")),
        "play.gave" => format!("The party gave {} to {}.", p["commodity"].as_str().unwrap_or(""), name("to")),
        "play.stole" => format!("The party tried to take {} from {}.", p["commodity"].as_str().unwrap_or(""), name("from")),
        "play.cast" => format!("A working of {} (level {}).", p["effect"].as_str().unwrap_or(""), p["level"].as_i64().unwrap_or(0)),
        "play.checked" => format!("{} check.", p["skill"].as_str().unwrap_or("")),
        "combat.started" => "The party started a fight.".into(),
        "combat.acted" => format!("Combat: {}.", p["action"].as_str().unwrap_or("")),
        "npc.edited" => format!("[GM] Edited {}.", name("person")),
        "org.policy_set" => format!("[GM] {} policy {} = {}.", name("org"), p["key"].as_str().unwrap_or(""), p["value"]),
        "relation.grievance" => format!("[GM] {} now holds a grievance against {}.", name("from"), name("to")),
        "magic.source_opened" => format!("An opening to another leaf changed the world's potential by {} thaums.", p["amount_milli"].as_i64().unwrap_or(0) / M),
        "npc.action" => format!("[GM] {} acts: {}.", name("actor"), p["action"].as_str().unwrap_or("")),
        _ => event_type.to_string(),
    }
}

pub fn apply(s: &mut WorldState, event_type: &str, p: &Value) -> Res<Vec<Incident>> {
    let mut out = vec![];
    if event_type != "world.genesis" && s.meta.world_id.is_empty() {
        return reject("no_world", "world has no genesis event");
    }
    match event_type {
        "world.genesis" => {
            if !s.meta.world_id.is_empty() {
                return reject("already_exists", "world already has a genesis event");
            }
            let seed = p.get("seed").and_then(|v| v.as_u64()).ok_or(Reject { code: "invalid_payload".into(), message: "missing seed".into() })?;
            *s = crate::worldgen::generate(gs(p, "world_id")?, gs(p, "name")?, seed);
        }
        "sim.tick" => out = step_day(s),
        "campaign.configured" => {
            let rules_id = gs(p, "rules")?;
            if !rules::PROFILES.contains(&rules_id) {
                return reject("invalid_rules", "rules must be srd-5.1 or srd-5.2.1");
            }
            if let Some(c) = &s.campaign {
                // A campaign never silently switches edition after it starts (GDD anti-pattern).
                if c.rules != rules_id && !s.party.members.is_empty() {
                    return reject("rules_locked", format!("this campaign is pinned to {}", c.rules));
                }
            }
            let gm = gs(p, "gm_mode")?;
            if !["human", "ai", "collaborative"].contains(&gm) {
                return reject("invalid_payload", "gm_mode must be human, ai or collaborative");
            }
            let mut content = std::collections::BTreeMap::new();
            if let Some(m) = p.get("content").and_then(|v| v.as_object()) {
                for (k, v) in m {
                    let lvl = v.as_str().unwrap_or("allowed");
                    if !["excluded", "veil", "limited", "allowed"].contains(&lvl) {
                        return reject("invalid_payload", "content levels: excluded, veil, limited, allowed");
                    }
                    content.insert(k.clone(), lvl.to_string());
                }
            }
            s.campaign = Some(Campaign { name: gs(p, "name")?.into(), rules: rules_id.into(), gm_mode: gm.into(), content });
        }
        "pc.created" => {
            if s.campaign.is_none() {
                return reject("no_campaign", "player characters belong to a campaign branch");
            }
            let name = gs(p, "name")?.trim().to_string();
            if name.is_empty() || name.len() > 40 {
                return reject("invalid_payload", "name must be 1..40 characters");
            }
            let class = gs(p, "class")?;
            let abilities = p.get("abilities").and_then(|v| v.as_array()).and_then(|a| {
                let v: Vec<i64> = a.iter().filter_map(|x| x.as_i64()).collect();
                <[i64; 6]>::try_from(v).ok()
            });
            let stats = rules::make_pc(class, abilities)?;
            let id = s.next_id("pc");
            let hh = "party".to_string();
            if !s.households.contains_key(&hh) {
                s.households.insert(hh.clone(), Household { id: hh.clone(), name: "The Party".into(), head: id.clone(), members: vec![], home: "loc_tavern".into(), dissolved: false, last_gouge_grievance: -1000 });
            }
            s.households.get_mut(&hh).unwrap().members.push(id.clone());
            let (first, family) = name.split_once(' ').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((name.clone(), "of No House".into()));
            s.persons.insert(id.clone(), Person { id: id.clone(), name: first, family, sex: p.get("sex").and_then(|v| v.as_str()).unwrap_or("x").into(), birth_day: s.meta.day - 25 * DAYS_PER_YEAR, status: "alive".into(), household: hh.clone(), home: "loc_tavern".into(), work: "loc_tavern".into(), occupation: "adventurer".into(), tier: 4, health: 100, stats, is_pc: true, agent_profile: "player".into(), languages: vec!["Sabline".into()], ..Default::default() });
            s.party.members.push(id.clone());
            s.issue(&hh, "coin", 60 * M); // starting purse: explicit, logged issuance
            s.issue(&hh, "grain", 10 * M);
        }
        "weather.shock" => {
            let kind = gs(p, "kind")?;
            if !["drought", "flood", "frost"].contains(&kind) {
                return reject("invalid_payload", "kind must be drought, flood or frost");
            }
            let days = gi(p, "days")?;
            let severity = p.get("severity").and_then(|v| v.as_i64()).unwrap_or(70).clamp(1, 100);
            if !(1..=720).contains(&days) {
                return reject("invalid_payload", "days must be 1..720");
            }
            let until_day = s.meta.day + days;
            s.weather.shocks.push(Shock { kind: kind.into(), until_day, severity });
        }
        "asset.destroyed" => {
            let id = gs(p, "asset_id")?;
            let Some(a) = s.assets.get_mut(id) else { return reject("unknown_entity", format!("no asset '{id}'")) };
            if !a.active {
                return reject("asset_inactive", format!("{} is already destroyed", a.name));
            }
            if a.protected {
                return reject("protected", format!("{} is protected canon and cannot be destroyed", a.name));
            }
            a.active = false;
            a.condition = 0;
        }
        "asset.sold" => {
            let id = gs(p, "asset_id")?;
            let buyer = gs(p, "buyer")?;
            let price = gi(p, "price_milli")?;
            let Some(a) = s.assets.get(id).cloned() else { return reject("unknown_entity", format!("no asset '{id}'")) };
            if !a.active {
                return reject("asset_inactive", format!("{} no longer exists and cannot be sold", a.name));
            }
            if let Some(seller) = p.get("seller").and_then(|v| v.as_str()) {
                if seller != a.owner {
                    return reject("not_owner", format!("{} is owned by {}, not {}", a.name, s.owner_name(&a.owner), s.owner_name(seller)));
                }
            }
            if !(s.households.contains_key(buyer) || s.orgs.contains_key(buyer)) {
                return reject("unknown_entity", format!("no buyer '{buyer}'"));
            }
            s.transfer(buyer, &a.owner, "coin", price)?;
            s.assets.get_mut(id).unwrap().owner = buyer.to_string();
        }
        "resource.issued" => {
            let amount = gi(p, "amount_milli")?;
            if amount <= 0 {
                return reject("invalid_amount", "issuance must be positive");
            }
            let owner = gs(p, "owner")?;
            if gs(p, "reason")?.trim().is_empty() {
                return reject("invalid_payload", "issuance requires an explicit reason");
            }
            if !(s.households.contains_key(owner) || s.orgs.contains_key(owner) || owner == "hinterland") {
                return reject("unknown_entity", format!("no account owner '{owner}'"));
            }
            s.issue(owner, gs(p, "commodity")?, amount);
        }
        "resource.transferred" => {
            let (from, to) = (gs(p, "from")?, gs(p, "to")?);
            for o in [from, to] {
                if !(s.households.contains_key(o) || s.orgs.contains_key(o) || o == "hinterland") {
                    return reject("unknown_entity", format!("no account owner '{o}'"));
                }
            }
            s.transfer(from, to, gs(p, "commodity")?, gi(p, "amount_milli")?)?;
        }
        "fact.recorded" => {
            let id = p.get("fact_id").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| s.next_id("fact"));
            if s.facts.contains_key(&id) {
                return reject("already_exists", format!("fact '{id}' already recorded{}", if s.facts[&id].protected { " and is protected" } else { "" }));
            }
            let day = s.meta.day;
            s.facts.insert(id.clone(), Fact { id, day, text: gs(p, "text")?.into(), protected: p.get("protected").and_then(|v| v.as_bool()).unwrap_or(false), gm_only: p.get("gm_only").and_then(|v| v.as_bool()).unwrap_or(false) });
        }
        "fact.protected" => {
            let id = gs(p, "fact_id")?;
            let Some(f) = s.facts.get_mut(id) else { return reject("unknown_entity", "no such fact") };
            f.protected = true;
        }
        "person.killed" => {
            let id = gs(p, "person")?;
            let Some(q) = s.persons.get(id) else { return reject("unknown_entity", format!("no person '{id}'")) };
            if !q.alive() {
                return reject("not_alive", format!("{} is not alive on this branch ({})", q.full_name(), q.status));
            }
            let cause = gs(p, "cause")?.to_string();
            kill(s, id, &cause, &mut out);
        }
        "npc.edited" => {
            let id = gs(p, "person")?;
            let Some(q) = s.persons.get_mut(id) else { return reject("unknown_entity", "no such person") };
            if let Some(t) = p.get("tier").and_then(|v| v.as_i64()) {
                if !(1..=4).contains(&t) {
                    return reject("invalid_payload", "tier must be 1..4");
                }
                // Promotion adds detail; it never rewrites identity, family or memories (AT-012).
                q.tier = t as u8;
                if !q.agent_profile.starts_with("custom") {
                    q.agent_profile = match t {
                        4 => "t4-world-shaper",
                        3 => "t3-major",
                        2 => "t2-recurring",
                        _ => "t1-resident",
                    }
                    .into();
                }
            }
            if let Some(a) = p.get("agent_profile").and_then(|v| v.as_str()) {
                q.agent_profile = a.to_string();
            }
            if let Some(g) = p.get("goals").and_then(|v| v.as_array()) {
                q.goals = g.iter().filter_map(|x| x.as_str().map(String::from)).take(6).collect();
            }
        }
        "org.policy_set" => {
            let org = gs(p, "org")?;
            let key = gs(p, "key")?.to_string();
            let v = gi(p, "value")?;
            let Some(o) = s.orgs.get_mut(org) else { return reject("unknown_entity", "no such organisation") };
            if !o.policy.contains_key(&key) {
                return reject("invalid_payload", format!("{} has no policy '{key}'", o.name));
            }
            o.policy.insert(key, v);
        }
        "relation.grievance" => {
            let (from, to) = (gs(p, "from")?.to_string(), gs(p, "to")?.to_string());
            if !s.persons.contains_key(&from) || !s.persons.contains_key(&to) {
                return reject("unknown_entity", "no such person");
            }
            let w = gi(p, "weight")?.clamp(1, 80);
            add_grievance(s, &from, &to, gs(p, "what")?, w, &mut out);
        }
        "magic.source_opened" => {
            // The only way potential enters or leaves the world: an explicit, owner-authorised exception (AT-019).
            let amt = gi(p, "amount_milli")?;
            if gs(p, "reason")?.trim().is_empty() {
                return reject("invalid_payload", "an opening requires a recorded reason");
            }
            if s.magic.ambient + amt < 0 {
                return reject("insufficient_potential", "cannot drain more than the ambient field holds");
            }
            s.magic.ambient += amt;
            s.magic.capacity_total += amt;
        }
        "npc.action" => {
            // Bounded, enumerated NPC proposals (from the GM or an AI agent). Anything else is rejected.
            let actor = gs(p, "actor")?.to_string();
            let Some(a) = s.persons.get(&actor).cloned() else { return reject("unknown_entity", "no such actor") };
            if !a.alive() || a.is_pc {
                return reject("invalid_actor", "actor must be a living NPC");
            }
            let day = s.meta.day;
            match gs(p, "action")? {
                "gift" => {
                    let to = gs(p, "target")?.to_string();
                    let Some(t) = s.persons.get(&to).cloned() else { return reject("unknown_entity", "no such target") };
                    let amount = gi(p, "amount_milli")?.clamp(0, 50 * M);
                    s.transfer(&a.household, &t.household, "coin", amount)?;
                    let r = s.rel_mut(&to, &actor);
                    r.fondness = (r.fondness + 6).min(100);
                    r.trust = (r.trust + 4).min(100);
                    inc(&mut out, "npc.gift", true, format!("{} made a gift of {} marks to {}.", a.full_name(), amount / M, t.full_name()), json!({"from": actor, "to": to}));
                }
                "reconcile" => {
                    let to = gs(p, "target")?.to_string();
                    if !s.persons.contains_key(&to) {
                        return reject("unknown_entity", "no such target");
                    }
                    let r = s.rel_mut(&actor, &to);
                    for g in r.grievances.iter_mut() {
                        g.weight /= 3;
                    }
                    crate::sim::social::update_status(s, &actor, &to, &mut out);
                    inc(&mut out, "relation.reconciled", true, format!("{} has made peace with {}.", a.full_name(), s.person_name(&to)), json!({"from": actor, "to": to}));
                }
                "set_goal" => {
                    let goal = gs(p, "goal")?.chars().take(160).collect::<String>();
                    let q = s.persons.get_mut(&actor).unwrap();
                    q.goals.insert(0, goal.clone());
                    q.goals.truncate(6);
                    q.remember(day, "resolve", format!("I have resolved: {goal}"), 70, vec![]);
                }
                "denounce" => {
                    let to = gs(p, "target")?.to_string();
                    if !s.persons.contains_key(&to) {
                        return reject("unknown_entity", "no such target");
                    }
                    add_grievance(s, &actor, &to, gs(p, "what")?, 25, &mut out);
                }
                "wait" => {}
                other => return reject("invalid_action", format!("'{other}' is not a permitted NPC action")),
            }
        }
        "merge.marker" => {}
        t if t.starts_with("play.") || t.starts_with("combat.") => {
            if s.campaign.is_none() {
                return reject("no_campaign", "play happens on a campaign branch");
            }
            out = crate::play::apply(s, t, p)?;
        }
        other => return reject("unknown_event", format!("unknown event type '{other}'")),
    }
    Ok(out)
}
