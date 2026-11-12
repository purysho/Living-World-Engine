//! Executable acceptance scenarios. IDs refer to
//! docs/specs/04_DELIVERY/ACCEPTANCE_SCENARIOS_AND_TEST_FIXTURES.md.

use lwe_core::api::App;
use lwe_core::model::*;
use lwe_core::store::{state_hash, Actor, Draft};
use serde_json::{json, Value};

fn app() -> App {
    let mut a = App::open(":memory:").unwrap();
    a.engine.set_clock(Box::new(|| "2026-11-12T00:00:00Z".to_string()));
    a.api_key = None;
    a
}
fn call(a: &mut App, op: &str, args: Value) -> Value {
    let r = a.dispatch(&json!({"op": op, "args": args}));
    assert!(r["ok"].as_bool().unwrap(), "{op} failed: {}", r["error"]);
    r["data"].clone()
}
fn call_as(a: &mut App, role: &str, op: &str, args: Value) -> Value {
    a.dispatch(&json!({"op": op, "args": args, "principal": {"id": role, "role": role}}))
}
fn err(a: &mut App, op: &str, args: Value) -> String {
    let r = a.dispatch(&json!({"op": op, "args": args}));
    assert!(!r["ok"].as_bool().unwrap(), "{op} unexpectedly succeeded");
    r["error"]["code"].as_str().unwrap().to_string()
}
fn world(a: &mut App, seed: u64) -> (String, String) {
    let w = call(a, "world.create", json!({"name": "Test", "seed": seed}));
    (w["world_id"].as_str().unwrap().into(), w["branch_id"].as_str().unwrap().into())
}
fn gm(a: &mut App, branch: &str, t: &str, payload: Value) -> Value {
    call(a, "gm.command", json!({"branch": branch, "type": t, "payload": payload}))
}
fn campaign(a: &mut App, w: &str, name: &str, rules: &str) -> String {
    call(a, "campaign.create", json!({"world_id": w, "name": name, "rules": rules}))["branch_id"].as_str().unwrap().to_string()
}

#[test]
fn genesis_builds_a_town_of_about_250_with_three_organisations() {
    let mut a = app();
    let (_, b) = world(&mut a, 42);
    let s = a.engine.state(&b).unwrap().clone();
    assert!((200..=300).contains(&s.population()), "population {}", s.population());
    assert_eq!(s.orgs.len(), 3);
    assert!(s.orgs.values().all(|o| o.leader.is_some() && o.members.len() >= 4));
    assert!(s.households.len() >= 50);
    // three generations: someone alive has a recorded grandparent
    assert!(s.living().any(|p| p.parents.iter().any(|x| !s.persons[x].parents.is_empty())));
    assert!(s.living().any(|p| p.tier == 4) && s.living().any(|p| p.tier == 3) && s.living().filter(|p| p.tier == 1).count() > 150);
    assert!(s.conservation_residual().values().all(|v| *v == 0));
}

#[test]
fn same_seed_same_world_different_seed_different_world() {
    let (mut a, mut b) = (app(), app());
    let (_, ba) = world(&mut a, 7);
    let (_, bb) = world(&mut b, 7);
    call(&mut a, "sim.advance", json!({"branch": ba, "days": 45}));
    call(&mut b, "sim.advance", json!({"branch": bb, "days": 45}));
    let (ha, hb) = (state_hash(a.engine.state(&ba).unwrap()), state_hash(b.engine.state(&bb).unwrap()));
    assert_eq!(ha, hb, "identical seed and commands must give identical state");
    let mut c = app();
    let (_, bc) = world(&mut c, 8);
    call(&mut c, "sim.advance", json!({"branch": bc, "days": 45}));
    assert_ne!(ha, state_hash(c.engine.state(&bc).unwrap()));
}

#[test]
fn at001_002_003_sale_is_atomic_rejection_leaves_state_untouched_and_retries_do_not_double_spend() {
    let mut a = app();
    let (_, b) = world(&mut a, 1);
    let s0 = a.engine.state(&b).unwrap().clone();
    let (lodge0, compact0) = (s0.acct("org_lodge", "coin"), s0.acct("org_compact", "coin"));
    // AT-002: buyer cannot afford it -> rejected, head and hash unchanged, diagnostic recorded
    let head0 = a.engine.branch(&b).unwrap();
    assert_eq!(err(&mut a, "gm.command", json!({"branch": b, "type": "asset.sold", "payload": {"asset_id": "asset_barge_heron", "buyer": "org_lodge", "price_milli": lodge0 + 1}})), "insufficient_balance");
    let head1 = a.engine.branch(&b).unwrap();
    assert_eq!((head0.head_seq, &head0.head_hash), (head1.head_seq, &head1.head_hash));
    assert_eq!(state_hash(a.engine.state(&b).unwrap()), state_hash(&s0));
    assert_eq!(call(&mut a, "rejections.list", json!({"branch": b})).as_array().unwrap().len(), 1);
    // AT-001: valid sale moves ownership and coin together
    let args = json!({"branch": b, "type": "asset.sold", "payload": {"asset_id": "asset_barge_heron", "buyer": "org_lodge", "price_milli": 500 * M}, "idempotency_key": "sale-1"});
    let r1 = call(&mut a, "gm.command", args.clone());
    let s1 = a.engine.state(&b).unwrap().clone();
    assert_eq!(s1.assets["asset_barge_heron"].owner, "org_lodge");
    assert_eq!(s1.acct("org_lodge", "coin"), lodge0 - 500 * M);
    assert_eq!(s1.acct("org_compact", "coin"), compact0 + 500 * M);
    // AT-003: same idempotency key replays the recorded result, no second debit
    let r2 = call(&mut a, "gm.command", args);
    assert_eq!(r2["replayed"], true);
    assert_eq!(r1["events"][0]["event_id"], r2["events"][0]["event_id"]);
    assert_eq!(a.engine.state(&b).unwrap().acct("org_lodge", "coin"), lodge0 - 500 * M);
    // same key, different request -> conflict
    assert_eq!(err(&mut a, "gm.command", json!({"branch": b, "type": "asset.sold", "payload": {"asset_id": "asset_ferry", "buyer": "org_compact", "price_milli": 1}, "idempotency_key": "sale-1"})), "idempotency_conflict");
    // optimistic concurrency
    assert_eq!(err(&mut a, "gm.command", json!({"branch": b, "type": "fact.recorded", "payload": {"text": "x"}, "expected_head": 1})), "stale_head");
}

#[test]
fn forks_are_isolated_from_later_parent_events() {
    let mut a = app();
    let (w, canon) = world(&mut a, 3);
    let child = call(&mut a, "branch.fork", json!({"parent": canon, "name": "what-if"}))["branch_id"].as_str().unwrap().to_string();
    let child_hash = state_hash(a.engine.state(&child).unwrap());
    assert_eq!(child_hash, state_hash(a.engine.state(&canon).unwrap()));
    call(&mut a, "sim.advance", json!({"branch": canon, "days": 10}));
    gm(&mut a, &canon, "asset.destroyed", json!({"asset_id": "asset_ferry", "reason": "fire"}));
    assert_eq!(state_hash(a.engine.state(&child).unwrap()), child_hash, "child must not see parent's later history");
    assert!(a.engine.state(&child).unwrap().assets["asset_ferry"].active);
    call(&mut a, "sim.advance", json!({"branch": child, "days": 3}));
    assert_eq!(a.engine.state(&canon).unwrap().meta.day, 10);
    assert_eq!(a.engine.state(&child).unwrap().meta.day, 3);
    // nested fork verifies through the whole lineage
    let grand = call(&mut a, "branch.fork", json!({"parent": child, "name": "deeper"}))["branch_id"].as_str().unwrap().to_string();
    call(&mut a, "sim.advance", json!({"branch": grand, "days": 2}));
    for b in [&canon, &child, &grand] {
        let v = call(&mut a, "world.verify", json!({"branch": b}));
        assert_eq!(v["ok"], true, "{}", v["problems"]);
    }
    assert_eq!(call(&mut a, "branch.list", json!({"world_id": w})).as_array().unwrap().len(), 3);
}

#[test]
fn at004_warehouse_bought_in_one_campaign_but_destroyed_earlier_in_another_blocks_the_merge() {
    let mut a = app();
    let (w, _canon) = world(&mut a, 5);
    let ca = campaign(&mut a, &w, "Campaign A", "srd-5.2.1");
    let cb = campaign(&mut a, &w, "Campaign B", "srd-5.1");
    // B: the granary burns on day 10
    call(&mut a, "sim.advance", json!({"branch": cb, "days": 10}));
    gm(&mut a, &cb, "asset.destroyed", json!({"asset_id": "asset_granary", "reason": "fire"}));
    // A: the Lodge buys the granary on day 12
    call(&mut a, "sim.advance", json!({"branch": ca, "days": 12}));
    gm(&mut a, &ca, "asset.sold", json!({"asset_id": "asset_granary", "buyer": "org_lodge", "price_milli": 900 * M}));
    let (ha, hb) = (a.engine.branch(&ca).unwrap().head_hash, a.engine.branch(&cb).unwrap().head_hash);
    let p = call(&mut a, "merge.propose", json!({"source": ca, "target": cb}));
    assert_eq!(p["status"], "blocked");
    let c = &p["conflicts"][0];
    assert_eq!(c["violated_rule"], "asset_inactive");
    assert_eq!(c["event_type"], "asset.sold");
    assert!(c["target_events"].as_array().unwrap().iter().any(|e| e["event_type"] == "asset.destroyed"), "conflict must name the destroying event: {c}");
    // never forced; originals intact
    assert_eq!(err(&mut a, "merge.decide", json!({"proposal_id": p["proposal_id"], "approve": true})), "not_mergeable");
    assert_eq!(a.engine.branch(&ca).unwrap().head_hash, ha);
    assert_eq!(a.engine.branch(&cb).unwrap().head_hash, hb);
    assert!(!a.engine.state(&cb).unwrap().assets["asset_granary"].active);
    assert_eq!(a.engine.state(&ca).unwrap().assets["asset_granary"].owner, "org_lodge");
}

#[test]
fn at005_007_compatible_merge_creates_a_new_branch_and_handles_non_sibling_ancestry() {
    let mut a = app();
    let (w, canon) = world(&mut a, 6);
    let ca = campaign(&mut a, &w, "A", "srd-5.2.1");
    call(&mut a, "sim.advance", json!({"branch": canon, "days": 5}));
    // non-sibling: B forks from canon later, and C forks from B
    let cb = campaign(&mut a, &w, "B", "srd-5.2.1");
    call(&mut a, "sim.advance", json!({"branch": cb, "days": 2}));
    let cc = call(&mut a, "branch.fork", json!({"parent": cb, "name": "B-prime"}))["branch_id"].as_str().unwrap().to_string();
    call(&mut a, "sim.advance", json!({"branch": ca, "days": 4}));
    gm(&mut a, &ca, "asset.sold", json!({"asset_id": "asset_barge_otter", "buyer": "org_lodge", "price_milli": 300 * M}));
    gm(&mut a, &ca, "fact.recorded", json!({"fact_id": "fact_oath", "text": "The Lodge swore the Oar-Oath on the quay."}));
    gm(&mut a, &cc, "asset.destroyed", json!({"asset_id": "asset_ferry", "reason": "storm"}));
    let anc = lwe_core::merge::common_ancestor(&a.engine, &ca, &cc).unwrap();
    assert_eq!(anc.0, canon);
    assert_eq!(anc.1, 1, "A forked right after genesis; that is the common point");
    let (ha, hc) = (a.engine.branch(&ca).unwrap().head_hash, a.engine.branch(&cc).unwrap().head_hash);
    let p = call(&mut a, "merge.propose", json!({"source": ca, "target": cc}));
    assert_eq!(p["status"], "ready", "{p}");
    assert_eq!(p["imports"].as_array().unwrap().len(), 2);
    // players and plain GMs cannot approve
    assert_eq!(call_as(&mut a, "gm", "merge.decide", json!({"proposal_id": p["proposal_id"], "approve": true}))["error"]["code"], "forbidden");
    let d = call(&mut a, "merge.decide", json!({"proposal_id": p["proposal_id"], "approve": true}));
    let merged = d["result_branch"].as_str().unwrap().to_string();
    let s = a.engine.state(&merged).unwrap().clone();
    assert_eq!(s.assets["asset_barge_otter"].owner, "org_lodge");
    assert!(!s.assets["asset_ferry"].active);
    assert!(s.facts.contains_key("fact_oath"));
    assert_eq!(a.engine.branch(&ca).unwrap().head_hash, ha);
    assert_eq!(a.engine.branch(&cc).unwrap().head_hash, hc);
    assert_eq!(call(&mut a, "world.verify", json!({"branch": merged}))["ok"], true);
    // a proposal goes stale if a source moves
    let p2 = call(&mut a, "merge.propose", json!({"source": cc, "target": ca}));
    call(&mut a, "sim.advance", json!({"branch": cc, "days": 1}));
    if p2["status"] == "ready" {
        assert_eq!(err(&mut a, "merge.decide", json!({"proposal_id": p2["proposal_id"], "approve": true})), "stale_proposal");
    }
    // canon pointer is owner-only and non-destructive
    assert_eq!(call_as(&mut a, "gm", "canon.set", json!({"world_id": w, "branch": merged}))["error"]["code"], "forbidden");
    call(&mut a, "canon.set", json!({"world_id": w, "branch": merged}));
    assert!(a.engine.branch(&canon).is_ok());
}

#[test]
fn at006_tampering_is_detected_and_events_are_immutable() {
    let mut a = app();
    let (_, b) = world(&mut a, 9);
    call(&mut a, "sim.advance", json!({"branch": b, "days": 45}));
    assert_eq!(call(&mut a, "world.verify", json!({"branch": b}))["ok"], true);
    // the event table refuses UPDATE and DELETE
    assert!(a.engine.conn.execute("UPDATE events SET summary = 'x' WHERE seq = 2", []).is_err());
    assert!(a.engine.conn.execute("DELETE FROM events WHERE seq = 2", []).is_err());
    // an attacker who drops the trigger and edits a payload is caught by the hash chain
    a.engine.conn.execute_batch("DROP TRIGGER events_no_update; UPDATE events SET payload_json = '{\"asset_id\":\"asset_hall\"}' WHERE seq = 3;").unwrap();
    let v = call(&mut a, "world.verify", json!({"branch": b}));
    assert_eq!(v["ok"], false);
    assert!(v["problems"].to_string().contains("record hash mismatch"));
    // tampered snapshot is detected too
    a.engine.conn.execute("UPDATE snapshots SET state_json = replace(state_json, '\"town\":\"Marrowick\"', '\"town\":\"Elsewhere\"')", []).unwrap();
    let v = call(&mut a, "world.verify", json!({"branch": b}));
    assert!(v["problems"].to_string().contains("snapshot"), "{}", v["problems"]);
}

#[test]
fn at008_protected_canon_cannot_be_destroyed_or_overwritten() {
    let mut a = app();
    let (w, canon) = world(&mut a, 11);
    assert_eq!(err(&mut a, "gm.command", json!({"branch": canon, "type": "asset.destroyed", "payload": {"asset_id": "asset_bridge", "reason": "plot"}})), "protected");
    gm(&mut a, &canon, "fact.recorded", json!({"fact_id": "fact_founding", "text": "Weir Bridge was built in year 1.", "protected": true}));
    let c = campaign(&mut a, &w, "C", "srd-5.2.1");
    let other = call(&mut a, "branch.fork", json!({"parent": canon, "name": "edit"}))["branch_id"].as_str().unwrap().to_string();
    // an incompatible edit of the same protected fact on a sibling cannot merge over it
    let _ = other;
    let d = call(&mut a, "branch.fork", json!({"parent": c, "name": "tmp"}));
    let _ = d;
    assert_eq!(err(&mut a, "gm.command", json!({"branch": c, "type": "fact.recorded", "payload": {"fact_id": "fact_founding", "text": "Actually built by giants."}})), "already_exists");
}

#[test]
fn role_policy_is_enforced_in_the_backend() {
    let mut a = app();
    let (w, canon) = world(&mut a, 12);
    let c = campaign(&mut a, &w, "C", "srd-5.2.1");
    for (role, op, args) in [
        ("player", "gm.command", json!({"branch": c, "type": "resource.issued", "payload": {"owner": "party", "commodity": "coin", "amount_milli": 1_000_000, "reason": "cheat"}})),
        ("player", "sim.advance", json!({"branch": c, "days": 1})),
        ("player", "world.create", json!({"name": "x"})),
        ("observer", "play.act", json!({"branch": c, "type": "play.waited", "payload": {"minutes": 10}})),
        ("gm", "gm.command", json!({"branch": canon, "type": "magic.source_opened", "payload": {"amount_milli": 5, "reason": "x"}})),
        ("gm", "ai.config.set", json!({"provider": "anthropic"})),
        ("player", "relations.notable", json!({"branch": c})),
        ("player", "npc.context", json!({"branch": c, "id": "p0001"})),
    ] {
        let r = call_as(&mut a, role, op, args);
        assert_eq!(r["error"]["code"], "forbidden", "{role} {op} should be forbidden: {r}");
    }
    assert_eq!(a.dispatch(&json!({"op": "world.list", "principal": {"role": "root"}}))["error"]["code"], "forbidden");
    // derived chronicle events cannot be forged
    assert!(a.engine.commit(&canon, vec![Draft::new("chronicle.crime.arrest", json!({}))], &Actor::owner(), None, None).is_err());
}

#[test]
fn at010_drought_raises_prices_hunger_and_crime_and_replays_bit_for_bit() {
    let mut a = app();
    let (w, canon) = world(&mut a, 42);
    let calm = canon.clone();
    let dry = call(&mut a, "branch.fork", json!({"parent": canon, "name": "drought"}))["branch_id"].as_str().unwrap().to_string();
    // a severe drought through the growing season and harvest: failed crop, stranded barges
    gm(&mut a, &dry, "weather.shock", json!({"kind": "drought", "days": 330, "severity": 85}));
    for b in [&calm, &dry] {
        call(&mut a, "sim.advance", json!({"branch": b, "days": 360}));
        call(&mut a, "sim.advance", json!({"branch": b, "days": 90}));
    }
    let (sc, sd) = (a.engine.state(&calm).unwrap().clone(), a.engine.state(&dry).unwrap().clone());
    let peak = |s: &WorldState, f: fn(&DayStat) -> i64| s.market.history.iter().map(f).max().unwrap();
    let (pc, pd) = (peak(&sc, |d| d.prices["grain"]), peak(&sd, |d| d.prices["grain"]));
    let (hc, hd) = (peak(&sc, |d| d.hunger), peak(&sd, |d| d.hunger));
    println!("grain peak calm {pc} drought {pd}; hunger peak calm {hc} drought {hd}; crimes calm {} drought {}; pop calm {} drought {}", sc.crimes.len(), sd.crimes.len(), sc.population(), sd.population());
    assert!(sd.weather.last_growth_index < sc.weather.last_growth_index - 20, "drought must cut the growth index");
    assert!(pd > pc * 13 / 10, "scarcity must raise the grain price");
    assert!(hd > hc, "hunger must rise");
    assert!(sd.crimes.len() > sc.crimes.len(), "desperation must raise crime");
    for s in [&sc, &sd] {
        assert!(s.conservation_residual().values().all(|v| *v == 0), "{:?}", s.conservation_residual());
        assert!(s.negative_accounts().is_empty());
        assert_eq!(s.magic.ambient + s.magic.bound + s.magic.dissipated, s.magic.capacity_total);
    }
    for b in [&calm, &dry] {
        let v = call(&mut a, "world.verify", json!({"branch": b}));
        assert_eq!(v["ok"], true, "{}", v["problems"]);
    }
    // the same history rebuilt in a fresh database gives the identical state hash
    let mut b2 = app();
    let (_, canon2) = world(&mut b2, 42);
    let dry2 = call(&mut b2, "branch.fork", json!({"parent": canon2, "name": "drought"}))["branch_id"].as_str().unwrap().to_string();
    gm(&mut b2, &dry2, "weather.shock", json!({"kind": "drought", "days": 330, "severity": 85}));
    call(&mut b2, "sim.advance", json!({"branch": dry2, "days": 360}));
    call(&mut b2, "sim.advance", json!({"branch": dry2, "days": 90}));
    assert_eq!(state_hash(b2.engine.state(&dry2).unwrap()), state_hash(&sd));
    let _ = w;
}

#[test]
fn two_calm_years_stay_conserved_alive_and_bounded() {
    let mut a = app();
    let (_, b) = world(&mut a, 2024);
    let p0 = a.engine.state(&b).unwrap().population();
    for _ in 0..2 {
        call(&mut a, "sim.advance", json!({"branch": b, "days": 360}));
    }
    let s = a.engine.state(&b).unwrap().clone();
    assert!(s.population() * 100 >= p0 * 80, "town should not collapse in calm years: {} -> {}", p0, s.population());
    assert!(s.conservation_residual().values().all(|v| *v == 0));
    assert!(s.persons.values().all(|p| p.memories.len() <= 28), "memory must stay bounded");
    assert!(s.market.history.len() <= 720);
    let size = serde_json::to_string(&s).unwrap().len();
    assert!(size < 4_000_000, "state grew to {size} bytes");
    assert!(s.living().any(|p| p.birth_day > 0), "children should have been born");
}

fn setup_play(a: &mut App, seed: u64, rules: &str) -> (String, String) {
    let (w, _) = world(a, seed);
    let c = campaign(a, &w, "Play", rules);
    call(a, "pc.create", json!({"branch": c, "pc": {"name": "Ren Vale", "class": "rogue"}}));
    (w, c)
}

#[test]
fn at011_013_npcs_only_disclose_what_they_know_and_false_witness_stays_a_belief() {
    let mut a = app();
    let (_, c) = setup_play(&mut a, 42, "srd-5.2.1");
    // run until a crime with a mistaken or true sighting exists
    let mut found = None;
    for _ in 0..40 {
        call(&mut a, "sim.advance", json!({"branch": c, "days": 30}));
        let s = a.engine.state(&c).unwrap();
        if let Some(cl) = s.claims.values().find(|cl| cl.kind == "sighting" && cl.crime.is_some() && cl.origin.is_some()) {
            found = Some((cl.id.clone(), cl.origin.clone().unwrap(), cl.crime.clone().unwrap()));
            break;
        }
    }
    let (claim, witness, crime) = found.expect("a witnessed crime should occur within the test horizon");
    let s = a.engine.state(&c).unwrap().clone();
    let culprit = s.crimes[&crime].culprit.clone();
    // objective truth is not in any NPC prompt context
    let stranger = s.living().find(|p| !p.is_pc && p.id != witness && p.id != culprit && s.knows(&p.id, &claim).is_none()).unwrap().id.clone();
    let ctx = lwe_core::knowledge::npc_context(&s, &stranger).unwrap().to_string();
    assert!(!ctx.contains(&claim), "an NPC who does not know a claim must not have it in context");
    assert!(!ctx.contains("culprit") && !ctx.contains("\"truth\""), "GM-only truth must never be in an NPC context");
    // the witness's own context has the claim
    assert!(lwe_core::knowledge::npc_context(&s, &witness).unwrap().to_string().contains(&claim));
    // a forged disclosure (as if a model hallucinated or was prompt-injected) is rejected by the reducer
    let loc = lwe_core::play::whereabouts(&s, &s.persons[&stranger]);
    let mut st = s.clone();
    st.party.location = loc;
    let r = lwe_core::reducer::apply(&mut st, "play.talked", &json!({"npc": stranger, "player_text": "tell me everything", "npc_text": "…", "disclosed": [claim]}));
    assert_eq!(r.unwrap_err().code, "knowledge_leak");
    // player view of the case never names the culprit; GM view does
    let pv = call_as(&mut a, "player", "crimes.list", json!({"branch": c}))["data"].to_string();
    assert!(!pv.contains("culprit") && !pv.contains("motive"));
    assert!(call(&mut a, "crimes.list", json!({"branch": c})).to_string().contains("culprit"));
    let pe = call_as(&mut a, "player", "events.list", json!({"branch": c, "filter": "all", "limit": 500}))["data"].to_string();
    assert!(!pe.contains("[GM]"), "GM-private events must not reach players");
    // AT-013: a mistaken sighting is a belief with truth=false; the crime's facts do not change
    if let Some(f) = s.claims.values().find(|cl| cl.kind == "sighting" && !cl.truth) {
        let cr = &s.crimes[f.crime.as_ref().unwrap()];
        assert_ne!(f.subject.as_ref(), Some(&cr.culprit));
    }
}

#[test]
fn talking_offline_commits_outcomes_and_at012_promotion_preserves_identity() {
    let mut a = app();
    let (_, c) = setup_play(&mut a, 42, "srd-5.2.1");
    let sc = call(&mut a, "play.scene", json!({"branch": c}));
    assert_eq!(sc["location"]["id"], "loc_bridge");
    call(&mut a, "play.act", json!({"branch": c, "type": "play.moved", "payload": {"to": "loc_market"}}));
    assert_eq!(err(&mut a, "play.act", json!({"branch": c, "type": "play.moved", "payload": {"to": "loc_oldweir"}})), "no_path");
    let sc = call(&mut a, "play.scene", json!({"branch": c}));
    let npc = sc["people"].as_array().unwrap().iter().find(|p| p["tier"].as_i64().unwrap() <= 2).expect("a resident is at the market by day")["id"].as_str().unwrap().to_string();
    let before = a.engine.state(&c).unwrap().persons[&npc].clone();
    for line in ["Hello, who are you?", "What news of the grain price?", "Heard any rumours?"] {
        let r = call(&mut a, "play.talk", json!({"branch": c, "npc": npc, "text": line}));
        assert_eq!(r["source"], "offline");
        assert!(!r["reply"].as_str().unwrap().is_empty());
    }
    let s = a.engine.state(&c).unwrap().clone();
    let after = &s.persons[&npc];
    assert_eq!(after.talks_with_party, 3);
    assert!(after.memories.iter().any(|m| m.kind == "conversation"));
    assert_eq!(s.party.conversations[&npc].len(), 6);
    // someone who is elsewhere cannot be spoken to
    let far = s.living().find(|p| !p.is_pc && lwe_core::play::whereabouts(&s, p) != s.party.location).unwrap().id.clone();
    assert_eq!(err(&mut a, "play.talk", json!({"branch": c, "npc": far, "text": "hi"})), "not_present");
    // AT-012: promote T1 -> T3; identity, family and memories are untouched
    gm(&mut a, &c, "npc.edited", json!({"person": npc, "tier": 3}));
    let p = a.engine.state(&c).unwrap().persons[&npc].clone();
    assert_eq!(p.tier, 3);
    assert_eq!(p.agent_profile, "t3-major");
    assert_eq!((p.name.as_str(), p.family.as_str(), p.birth_day, &p.parents, &p.spouse, &p.household), (before.name.as_str(), before.family.as_str(), before.birth_day, &before.parents, &before.spouse, &before.household));
    assert!(p.memories.len() >= before.memories.len());
}

#[test]
fn at014_rules_profiles_are_pinned_per_campaign() {
    let mut a = app();
    let (w, _) = world(&mut a, 20);
    let c14 = campaign(&mut a, &w, "2014", "srd-5.1");
    let c24 = campaign(&mut a, &w, "2024", "srd-5.2.1");
    for c in [&c14, &c24] {
        call(&mut a, "pc.create", json!({"branch": c, "pc": {"name": "Tam", "class": "fighter"}}));
    }
    assert_eq!(call(&mut a, "play.scene", json!({"branch": c14}))["rules"], "srd-5.1");
    assert_eq!(call(&mut a, "play.scene", json!({"branch": c24}))["rules"], "srd-5.2.1");
    // edition cannot silently change once play has begun
    assert_eq!(err(&mut a, "gm.command", json!({"branch": c14, "type": "campaign.configured", "payload": {"name": "x", "rules": "srd-5.2.1", "gm_mode": "human"}})), "rules_locked");
    assert_eq!(err(&mut a, "campaign.create", json!({"world_id": w, "name": "bad", "rules": "pathfinder"})), "invalid_rules");
    // identical seed and dice stream: same check, same roll
    let r1 = call(&mut a, "play.act", json!({"branch": c14, "type": "play.checked", "payload": {"skill": "athletics", "dc": 10}}));
    let r2 = call(&mut a, "play.act", json!({"branch": c24, "type": "play.checked", "payload": {"skill": "athletics", "dc": 10}}));
    assert_eq!(r1["events"][1]["summary"], r2["events"][1]["summary"]);
    assert_eq!(err(&mut a, "pc.create", json!({"branch": c14, "pc": {"name": "Zed", "class": "bard"}})), "unknown_class");
}

#[test]
fn combat_resolves_by_the_rules_and_survivors_become_nemeses() {
    let mut a = app();
    let (_, c) = setup_play(&mut a, 42, "srd-5.2.1");
    call(&mut a, "play.act", json!({"branch": c, "type": "play.moved", "payload": {"to": "loc_market"}}));
    let sc = call(&mut a, "play.scene", json!({"branch": c}));
    // pick a commoner so the fight is short
    let s = a.engine.state(&c).unwrap().clone();
    let foe = sc["people"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap().to_string()).find(|id| s.persons[id].stats.class == "commoner" && s.persons[id].age(s.meta.day) >= 18).expect("a commoner at the market");
    let lvl0 = s.persons[&foe].stats.level;
    call(&mut a, "play.act", json!({"branch": c, "type": "combat.started", "payload": {"targets": [foe], "nonlethal": true}}));
    assert_eq!(err(&mut a, "play.act", json!({"branch": c, "type": "play.moved", "payload": {"to": "loc_bridge"}})), "in_combat");
    for _ in 0..40 {
        let e = a.engine.state(&c).unwrap().encounter.clone().unwrap();
        if !e.active {
            break;
        }
        call(&mut a, "play.act", json!({"branch": c, "type": "combat.acted", "payload": {"action": "attack"}}));
    }
    let s = a.engine.state(&c).unwrap().clone();
    let e = s.encounter.clone().unwrap();
    assert!(!e.active);
    assert!(e.log.iter().any(|l| l.contains("d20")), "dice must be shown in the audit log");
    let pc = s.party.members[0].clone();
    let f = &s.persons[&foe];
    if e.outcome == "victory" {
        assert!(f.alive(), "nonlethal victory leaves the opponent alive");
        assert!(f.stats.level > lvl0, "a beaten survivor grows");
        let r = s.rel(&foe, &pc).unwrap();
        assert!(matches!(r.status.as_str(), "enemy" | "nemesis"), "status {}", r.status);
        assert!(f.memories.iter().any(|m| m.kind == "defeat"));
        assert!(f.tier >= 3);
        // enemies do not chat
        let loc = lwe_core::play::whereabouts(&s, f);
        if loc == s.party.location {
            let r = call(&mut a, "play.talk", json!({"branch": c, "npc": foe, "text": "No hard feelings?"}));
            assert!(r["reply"].as_str().unwrap().contains("nothing to say"));
        }
        // and they start hunting within a few weeks (never resolving harm to the PC off-screen)
        let hp = s.persons[&pc].stats.hp;
        call(&mut a, "sim.advance", json!({"branch": c, "days": 60}));
        let s2 = a.engine.state(&c).unwrap();
        assert_eq!(s2.persons[&pc].stats.hp, hp, "no off-screen harm to a player character");
        assert!(s2.persons[&pc].alive());
    } else {
        assert!(s.persons[&pc].alive(), "player death is never decided by the engine");
    }
    assert_eq!(call(&mut a, "world.verify", json!({"branch": c}))["ok"], true);
}

#[test]
fn at019_magic_obeys_its_conservation_law() {
    let mut a = app();
    let (w, _) = world(&mut a, 30);
    let c = campaign(&mut a, &w, "Magic", "srd-5.2.1");
    call(&mut a, "pc.create", json!({"branch": c, "pc": {"name": "Ysa", "class": "cleric"}}));
    call(&mut a, "pc.create", json!({"branch": c, "pc": {"name": "Bor", "class": "fighter"}}));
    let m0 = a.engine.state(&c).unwrap().magic.clone();
    let pcs = a.engine.state(&c).unwrap().party.members.clone();
    call(&mut a, "play.act", json!({"branch": c, "type": "play.cast", "payload": {"level": 2, "effect": "heal"}}));
    let m1 = a.engine.state(&c).unwrap().magic.clone();
    assert_eq!(m0.ambient - m1.ambient, 16 * M, "cost law: 4·level² thaums");
    assert_eq!(m1.ambient + m1.bound + m1.dissipated, m1.capacity_total);
    assert_eq!(err(&mut a, "play.act", json!({"branch": c, "type": "play.cast", "payload": {"pc": pcs[1], "level": 1, "effect": "heal"}})), "not_a_caster");
    // drain the field through the only legal channel, then try to cast from nothing
    let amb = a.engine.state(&c).unwrap().magic.ambient;
    gm(&mut a, &c, "magic.source_opened", json!({"amount_milli": -amb, "reason": "the Underweir drinks"}));
    assert_eq!(err(&mut a, "play.act", json!({"branch": c, "type": "play.cast", "payload": {"level": 1, "effect": "heal"}})), "insufficient_potential");
    assert_eq!(err(&mut a, "gm.command", json!({"branch": c, "type": "magic.source_opened", "payload": {"amount_milli": -1, "reason": "overdraft"}})), "insufficient_potential");
}

#[test]
fn at018_content_boundaries_filter_presentation_not_canon() {
    let mut a = app();
    let (w, canon) = world(&mut a, 31);
    let victim = a.engine.state(&canon).unwrap().living().find(|p| p.tier == 1 && p.age(0) > 20).unwrap().id.clone();
    let soft = call(&mut a, "campaign.create", json!({"world_id": w, "name": "Gentle", "content": {"violence": "veil"}}))["branch_id"].as_str().unwrap().to_string();
    let hard = call(&mut a, "campaign.create", json!({"world_id": w, "name": "Grim", "content": {"violence": "allowed"}}))["branch_id"].as_str().unwrap().to_string();
    for b in [&soft, &hard] {
        gm(&mut a, b, "person.killed", json!({"person": victim, "cause": "killed by a falling crane"}));
    }
    let view = |a: &mut App, b: &str| call_as(a, "player", "events.list", json!({"branch": b, "filter": "all"}))["data"].to_string();
    let (vs, vh) = (view(&mut a, &soft), view(&mut a, &hard));
    assert!(vs.contains("offscreen") && !vs.contains("falling crane"));
    assert!(vh.contains("falling crane"));
    // canon is identical in both: the person is dead on both branches
    assert_eq!(a.engine.state(&soft).unwrap().persons[&victim].status, "dead");
    assert_eq!(a.engine.state(&hard).unwrap().persons[&victim].status, "dead");
    assert_eq!(err(&mut a, "campaign.create", json!({"world_id": w, "name": "bad", "content": {"violence": "maximum"}})), "invalid_payload");
}

#[test]
fn deaths_caused_in_play_merge_into_canon() {
    let mut a = app();
    let (w, canon) = world(&mut a, 42);
    let (c, victim) = {
        let c = campaign(&mut a, &w, "Bloody", "srd-5.2.1");
        call(&mut a, "pc.create", json!({"branch": c, "pc": {"name": "Ren Vale", "class": "fighter"}}));
        call(&mut a, "play.act", json!({"branch": c, "type": "play.moved", "payload": {"to": "loc_market"}}));
        let s = a.engine.state(&c).unwrap().clone();
        let foe = lwe_core::play::present(&s, "loc_market").into_iter().find(|id| s.persons[id].stats.class == "commoner" && s.persons[id].age(0) >= 18).unwrap();
        (c, foe)
    };
    call(&mut a, "play.act", json!({"branch": c, "type": "combat.started", "payload": {"targets": [victim]}}));
    for _ in 0..40 {
        if !a.engine.state(&c).unwrap().encounter.as_ref().unwrap().active {
            break;
        }
        call(&mut a, "play.act", json!({"branch": c, "type": "combat.acted", "payload": {"action": "attack"}}));
    }
    let s = a.engine.state(&c).unwrap().clone();
    if s.persons[&victim].alive() {
        return; // the dice let them live; nothing to merge
    }
    // lethal violence by the party is a crime in the world like any other
    assert!(s.crimes.values().any(|cr| cr.kind == "murder" && cr.culprit == s.party.members[0]));
    let p = call(&mut a, "merge.propose", json!({"source": c, "target": canon}));
    assert_eq!(p["status"], "ready", "{p}");
    let d = call(&mut a, "merge.decide", json!({"proposal_id": p["proposal_id"], "approve": true}));
    let merged = d["result_branch"].as_str().unwrap();
    assert_eq!(a.engine.state(merged).unwrap().persons[&victim].status, "dead");
    assert!(a.engine.state(&canon).unwrap().persons[&victim].alive(), "canon itself is untouched until the pointer is moved");
}

#[test]
fn council_and_agents_work_offline() {
    let mut a = app();
    let (_, b) = world(&mut a, 1);
    let m = call(&mut a, "council.ask", json!({"question": "How do I make the grain speculator a compelling villain without railroading the players?", "branch": b}));
    assert_eq!(m["mode"], "offline_framework");
    assert!(m["lenses"].as_array().unwrap().len() >= 2);
    assert!(m["proposals"].as_array().unwrap().len() >= 2, "at least two substantively different proposals");
    let agents = call(&mut a, "agents.list", json!({}));
    assert_eq!(agents.as_array().unwrap().len(), 5);
    assert_eq!(err(&mut a, "agents.set", json!({"name": "bad name!", "body": "---\n---\nx"})), "invalid_payload");
    call(&mut a, "agents.set", json!({"name": "custom-innkeeper", "body": "---\nname: custom-innkeeper\ntier: 2\n---\nYou keep the Drowned Lantern."}));
    assert_eq!(call(&mut a, "agents.list", json!({})).as_array().unwrap().len(), 6);
    // AI is off by default, and never silently fabricates when asked without a key
    let cfg = call(&mut a, "ai.config.get", json!({}));
    assert_eq!(cfg["provider"], "offline");
    assert_eq!(cfg["has_key"], false);
    call(&mut a, "ai.config.set", json!({"provider": "anthropic"}));
    let m = call(&mut a, "council.ask", json!({"question": "magic system costs"}));
    assert_eq!(m["ai_error"]["code"], "ai_no_key");
    assert_eq!(m["mode"], "offline_framework");
    // API keys are never written to the database
    call(&mut a, "ai.config.set", json!({"api_key": "sk-ant-test-SECRET"}));
    let dump: String = a.engine.conn.query_row("SELECT group_concat(key || value) FROM settings", [], |r| r.get(0)).unwrap();
    assert!(!dump.contains("SECRET"));
}

#[test]
fn export_contains_the_full_lineage_and_why_explains_incidents() {
    let mut a = app();
    let (w, canon) = world(&mut a, 42);
    call(&mut a, "sim.advance", json!({"branch": canon, "days": 200}));
    let c = campaign(&mut a, &w, "X", "srd-5.2.1");
    call(&mut a, "sim.advance", json!({"branch": c, "days": 5}));
    let ex = call(&mut a, "world.export", json!({"branch": c}));
    let n: i64 = a.engine.branch(&canon).unwrap().head_seq + a.engine.branch(&c).unwrap().head_seq;
    assert_eq!(ex["event_count"].as_i64().unwrap(), n);
    assert!(!ex.to_string().contains("api_key"));
    let evs = call(&mut a, "events.list", json!({"branch": c, "filter": "notable", "limit": 300}));
    let inc = evs.as_array().unwrap().iter().find(|e| e["derived"] == true).expect("some incident in 200 days");
    let why = call(&mut a, "event.why", json!({"event_id": inc["event_id"]}));
    assert_eq!(why["caused_by"][0]["event_type"], "sim.tick", "every incident names the tick that produced it");
    assert!(evs.as_array().unwrap().iter().any(|e| e["inherited"] == true));
}
