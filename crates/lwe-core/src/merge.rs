//! Historical reconciliation (Decisions 01, 22, 27). A merge never rewrites
//! either source. It replays the *interventions* of one branch onto the head
//! of another, in fictional-time order, through the ordinary reducers; the
//! first transition that the target's state cannot legally accept is reported
//! as a conflict with the events involved. Conservative by design: a false
//! conflict is preferable to silent canon damage.

use crate::model::*;
use crate::reducer;
use crate::store::{sha, Actor, Draft, Engine, EventRecord};
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

fn db<T>(r: rusqlite::Result<T>) -> Res<T> {
    r.map_err(|e| Reject { code: "storage".into(), message: e.to_string() })
}

/// Lowest common ancestor of two branches as (branch, seq).
pub fn common_ancestor(e: &Engine, a: &str, b: &str) -> Res<(String, i64)> {
    let (la, lb) = (e.lineage(a)?, e.lineage(b)?);
    let mut best: Option<(String, i64)> = None;
    for ((ba, sa), (bb, sb)) in la.iter().zip(lb.iter()) {
        if ba != bb {
            break;
        }
        best = Some((ba.clone(), (*sa).min(*sb)));
        if sa != sb {
            break;
        }
    }
    best.ok_or(Reject { code: "unrelated".into(), message: "branches share no history".into() })
}

/// Events on `branch`'s lineage that happened after the common ancestor point.
fn delta(e: &Engine, branch: &str, anc: &(String, i64)) -> Res<Vec<EventRecord>> {
    let mut out = vec![];
    let mut seen_anc = false;
    for (bid, upto) in e.lineage(branch)? {
        if bid == anc.0 {
            seen_anc = true;
            out.extend(e.events_range(&bid, anc.1, upto)?);
        } else if seen_anc {
            out.extend(e.events_range(&bid, 0, upto)?);
        }
    }
    Ok(out)
}

/// What a source branch contributes: explicit world interventions, plus deaths that play caused.
fn imports(events: &[EventRecord]) -> Vec<(EventRecord, Draft)> {
    let mut v = vec![];
    for ev in events {
        if !ev.derived && reducer::mergeable(&ev.event_type) {
            v.push((ev.clone(), Draft::new(&ev.event_type, ev.payload.clone()).origin("merge").causes(vec![ev.event_id.clone()])));
        } else if ev.derived && ev.event_type == "chronicle.person.died" && ev.payload["cause"].as_str().map(|c| c.starts_with("killed by")).unwrap_or(false) {
            let parent_is_play = ev.causes.first().map(|c| events.iter().any(|x| &x.event_id == c && (x.event_type.starts_with("combat.") || x.event_type.starts_with("play."))) ).unwrap_or(false);
            if parent_is_play {
                v.push((ev.clone(), Draft::new("person.killed", json!({"person": ev.payload["person"], "cause": ev.payload["cause"]})).origin("merge").causes(vec![ev.event_id.clone()])));
            }
        }
    }
    v.sort_by_key(|(ev, _)| (ev.occurred_day, ev.branch_id.clone(), ev.seq));
    v
}

fn ids_in(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) if s.len() <= 40 && (s.starts_with("asset_") || s.starts_with("org_") || s.starts_with('p') && s[1..].chars().all(|c| c.is_ascii_digit()) || s.starts_with("fact") || s.starts_with('h') && s.len() > 1 && s[1..].chars().all(|c| c.is_ascii_digit())) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| ids_in(x, out)),
        Value::Object(o) => o.values().for_each(|x| ids_in(x, out)),
        _ => {}
    }
}

pub fn propose(e: &mut Engine, source: &str, target: &str, actor: &Actor) -> Res<Value> {
    if !["world_owner", "gm"].contains(&actor.role.as_str()) {
        return reject("forbidden", "only the World Owner or a GM can propose a merge");
    }
    let (sb, tb) = (e.branch(source)?, e.branch(target)?);
    if sb.world_id != tb.world_id || source == target {
        return reject("invalid_payload", "choose two different branches of the same world");
    }
    let anc = common_ancestor(e, source, target)?;
    let src_delta = delta(e, source, &anc)?;
    let tgt_delta = delta(e, target, &anc)?;
    let imp = imports(&src_delta);
    let mut state = e.state(target)?.clone();
    let mut conflicts: Vec<Value> = vec![];
    let mut plan: Vec<Value> = vec![];
    let mut ticks = 0;
    for (ev, d) in &imp {
        // an imported event cannot happen before the target's present: advance the target to that day first
        while state.meta.day < ev.occurred_day && ticks < 800 {
            reducer::apply(&mut state, "sim.tick", &json!({}))?;
            ticks += 1;
        }
        let mut trial = state.clone();
        match reducer::apply(&mut trial, &d.event_type, &d.payload) {
            Ok(_) => {
                state = trial;
                plan.push(json!({"source_event": ev.event_id, "event_type": d.event_type, "day": ev.occurred_day, "summary": ev.summary}));
            }
            Err(rej) => {
                let mut ids = vec![];
                ids_in(&d.payload, &mut ids);
                let related: Vec<Value> = tgt_delta.iter().filter(|t| !t.derived || t.event_type.contains("died") || t.event_type.contains("destroyed")).filter(|t| {
                    let mut tids = vec![];
                    ids_in(&t.payload, &mut tids);
                    tids.iter().any(|i| ids.contains(i))
                }).rev().take(5).map(|t| json!({"event_id": t.event_id, "day": t.occurred_day, "date": date_string(t.occurred_day), "event_type": t.event_type, "summary": t.summary})).collect();
                conflicts.push(json!({"source_event": ev.event_id, "event_type": d.event_type, "day": ev.occurred_day, "date": date_string(ev.occurred_day), "summary": ev.summary, "violated_rule": rej.code, "explanation": rej.message, "entities": ids, "target_events": related}));
            }
        }
    }
    let status = if conflicts.is_empty() { if plan.is_empty() { "empty" } else { "ready" } } else { "blocked" };
    // approval is bound to exactly these heads and this event set; any change makes it stale
    let digest = sha(format!("{}|{}|{}|{}", sb.head_hash, tb.head_hash, anc.1, plan.iter().map(|p| p["source_event"].as_str().unwrap_or("")).collect::<Vec<_>>().join(",")).as_bytes());
    let n: i64 = db(e.conn.query_row("SELECT COUNT(*) FROM proposals", [], |r| r.get(0)))?;
    let id = format!("merge_{:04}", n + 1);
    let report = json!({"proposal_id": id, "status": status, "source": source, "target": target, "common_ancestor": {"branch": anc.0, "seq": anc.1}, "source_events_considered": src_delta.iter().filter(|x| !x.derived).count(), "imports": plan, "conflicts": conflicts, "target_days_advanced": ticks, "digest": digest,
        "note": "Simulation ticks and play events are not replayed onto the target; only explicit world interventions and deaths caused in play are carried across."});
    db(e.conn.execute("INSERT INTO proposals VALUES (?,?,?,?,?,?,?,?,?,?,NULL,NULL)", params![id, sb.world_id, source, target, sb.head_hash, tb.head_hash, status, digest, report.to_string(), e.now()]))?;
    Ok(report)
}

pub fn list(e: &Engine, world: &str) -> Res<Vec<Value>> {
    let mut st = db(e.conn.prepare("SELECT report_json, status, decided_by, result_branch, created_at FROM proposals WHERE world_id = ? ORDER BY created_at DESC, proposal_id DESC"))?;
    let rows = db(st.query_map(params![world], |r| {
        let mut v: Value = serde_json::from_str(&r.get::<_, String>(0)?).unwrap_or(Value::Null);
        v["status"] = json!(r.get::<_, String>(1)?);
        v["decided_by"] = json!(r.get::<_, Option<String>>(2)?);
        v["result_branch"] = json!(r.get::<_, Option<String>>(3)?);
        v["created_at"] = json!(r.get::<_, String>(4)?);
        Ok(v)
    }))?;
    db(rows.collect())
}

pub fn decide(e: &mut Engine, proposal: &str, approve: bool, actor: &Actor) -> Res<Value> {
    if actor.role != "world_owner" {
        return reject("forbidden", "merge approval requires the World Owner");
    }
    let row: Option<(String, String, String, String, String, String)> = db(e.conn.query_row("SELECT source_branch, target_branch, source_head, target_head, status, report_json FROM proposals WHERE proposal_id = ?", params![proposal], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))).optional())?;
    let Some((source, target, sh, th, status, report)) = row else { return reject("unknown_entity", "no such proposal") };
    if !approve {
        db(e.conn.execute("UPDATE proposals SET status = 'rejected', decided_by = ? WHERE proposal_id = ?", params![actor.principal, proposal]))?;
        return Ok(json!({"proposal_id": proposal, "status": "rejected"}));
    }
    if status != "ready" {
        return reject("not_mergeable", format!("proposal is '{status}'; conflicts must be resolved in a new branch, never forced"));
    }
    if e.branch(&source)?.head_hash != sh || e.branch(&target)?.head_hash != th {
        db(e.conn.execute("UPDATE proposals SET status = 'stale' WHERE proposal_id = ?", params![proposal]))?;
        return reject("stale_proposal", "a source branch changed after this proposal was computed; propose again");
    }
    let report: Value = serde_json::from_str(&report).unwrap_or(Value::Null);
    let name = format!("Merge of {} into {}", e.branch(&source)?.name, e.branch(&target)?.name);
    let merged = e.fork(&target, &name, "merge", actor)?;
    let mut drafts = vec![Draft::new("merge.marker", json!({"proposal": proposal, "source": source, "target": target, "digest": report["digest"]})).origin("merge")];
    let mut day = e.state(&merged.branch_id)?.meta.day;
    for imp in report["imports"].as_array().cloned().unwrap_or_default() {
        let ev = e.event(imp["source_event"].as_str().unwrap_or(""))?;
        while day < ev.occurred_day {
            drafts.push(Draft::new("sim.tick", json!({})).origin("simulation"));
            day += 1;
        }
        let (et, payload) = if ev.derived { ("person.killed".to_string(), json!({"person": ev.payload["person"], "cause": ev.payload["cause"]})) } else { (ev.event_type.clone(), ev.payload.clone()) };
        drafts.push(Draft::new(&et, payload).origin("merge").causes(vec![ev.event_id.clone()]));
    }
    let c = e.commit(&merged.branch_id, drafts, actor, None, None)?;
    db(e.conn.execute("UPDATE proposals SET status = 'merged', decided_by = ?, result_branch = ? WHERE proposal_id = ?", params![actor.principal, merged.branch_id, proposal]))?;
    Ok(json!({"proposal_id": proposal, "status": "merged", "result_branch": merged.branch_id, "events_committed": c.events.len()}))
}
