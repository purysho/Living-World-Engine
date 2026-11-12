//! Append-only, hash-chained event store on SQLite (WAL, single local writer),
//! with branch lineage, snapshots, idempotency, optimistic concurrency and
//! full replay verification (Decisions 51–53, 55).
//!
//! Integrity note: hashes detect accidental or casual tampering. They are not
//! signatures and do not protect against someone who can rewrite the whole file.

use crate::model::*;
use crate::reducer;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const SNAPSHOT_EVERY: i64 = 40;

pub fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn state_hash(s: &WorldState) -> String {
    sha(&serde_json::to_vec(s).expect("state serialises"))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub principal: String,
    pub role: String,
}
impl Actor {
    pub fn owner() -> Self {
        Actor { principal: "local_owner".into(), role: "world_owner".into() }
    }
    pub fn system() -> Self {
        Actor { principal: "simulation".into(), role: "world_owner".into() }
    }
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub event_type: String,
    pub payload: Value,
    pub causes: Vec<String>,
    /// human | simulation | ai_proposal | merge | system
    pub origin: String,
}
impl Draft {
    pub fn new(event_type: &str, payload: Value) -> Self {
        Draft { event_type: event_type.into(), payload, causes: vec![], origin: "human".into() }
    }
    pub fn origin(mut self, o: &str) -> Self {
        self.origin = o.into();
        self
    }
    pub fn causes(mut self, c: Vec<String>) -> Self {
        self.causes = c;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub event_id: String,
    pub world_id: String,
    pub branch_id: String,
    pub seq: i64,
    pub event_type: String,
    pub event_version: i64,
    pub occurred_day: i64,
    pub committed_at: String,
    pub causes: Vec<String>,
    pub actor: Actor,
    pub origin: String,
    pub payload: Value,
    pub privacy: String,
    pub summary: String,
    pub prev_hash: String,
    pub record_hash: String,
    pub state_hash: String,
    pub derived: bool,
}

impl EventRecord {
    /// Canonical serialisation for hashing: every field except the record hash itself.
    fn compute_hash(&self) -> String {
        let v = json!({
            "schema": "lwe.event/1", "event_id": self.event_id, "world_id": self.world_id, "branch_id": self.branch_id,
            "seq": self.seq, "event_type": self.event_type, "event_version": self.event_version,
            "occurred_day": self.occurred_day, "committed_at": self.committed_at, "causes": self.causes,
            "actor": self.actor, "origin": self.origin, "payload": self.payload, "privacy": self.privacy,
            "summary": self.summary, "prev_hash": self.prev_hash, "state_hash": self.state_hash, "derived": self.derived,
        });
        sha(serde_json::to_string(&v).unwrap().as_bytes())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branch {
    pub branch_id: String,
    pub world_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub fork_seq: i64,
    pub head_seq: i64,
    pub head_hash: String,
    pub kind: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Committed {
    pub head_seq: i64,
    pub events: Vec<EventRecord>,
    pub replayed: bool,
}

pub struct Engine {
    pub conn: Connection,
    cache: HashMap<String, (i64, WorldState)>,
    clock: Box<dyn Fn() -> String + Send>,
}

fn db<T>(r: rusqlite::Result<T>) -> Res<T> {
    r.map_err(|e| Reject { code: "storage".into(), message: e.to_string() })
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    // civil-from-days (Howard Hinnant), UTC
    let days = (secs / 86400) as i64;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs % 86400 / 3600, secs % 3600 / 60, secs % 60)
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS worlds (world_id TEXT PRIMARY KEY, name TEXT NOT NULL, seed TEXT NOT NULL, created_at TEXT NOT NULL, canon_branch TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS branches (branch_id TEXT PRIMARY KEY, world_id TEXT NOT NULL REFERENCES worlds(world_id), name TEXT NOT NULL, parent_id TEXT REFERENCES branches(branch_id), fork_seq INTEGER NOT NULL, head_seq INTEGER NOT NULL DEFAULT 0, head_hash TEXT NOT NULL, kind TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS events (event_id TEXT PRIMARY KEY, world_id TEXT NOT NULL, branch_id TEXT NOT NULL REFERENCES branches(branch_id), seq INTEGER NOT NULL, event_type TEXT NOT NULL, event_version INTEGER NOT NULL, occurred_day INTEGER NOT NULL, committed_at TEXT NOT NULL, causes_json TEXT NOT NULL, actor_json TEXT NOT NULL, origin TEXT NOT NULL, payload_json TEXT NOT NULL, privacy TEXT NOT NULL, summary TEXT NOT NULL, prev_hash TEXT NOT NULL, record_hash TEXT NOT NULL, state_hash TEXT NOT NULL, derived INTEGER NOT NULL, UNIQUE(branch_id, seq));
CREATE TABLE IF NOT EXISTS event_causes (event_id TEXT NOT NULL, cause_event_id TEXT NOT NULL, relation TEXT NOT NULL, PRIMARY KEY(event_id, cause_event_id, relation));
CREATE INDEX IF NOT EXISTS idx_causes_source ON event_causes(cause_event_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(branch_id, event_type, seq);
CREATE TABLE IF NOT EXISTS idempotency (branch_id TEXT NOT NULL, key TEXT NOT NULL, request_hash TEXT NOT NULL, response_json TEXT NOT NULL, PRIMARY KEY(branch_id, key));
CREATE TABLE IF NOT EXISTS snapshots (branch_id TEXT NOT NULL, seq INTEGER NOT NULL, state_hash TEXT NOT NULL, state_json TEXT NOT NULL, PRIMARY KEY(branch_id, seq));
CREATE TABLE IF NOT EXISTS proposals (proposal_id TEXT PRIMARY KEY, world_id TEXT NOT NULL, source_branch TEXT NOT NULL, target_branch TEXT NOT NULL, source_head TEXT NOT NULL, target_head TEXT NOT NULL, status TEXT NOT NULL, digest TEXT NOT NULL, report_json TEXT NOT NULL, created_at TEXT NOT NULL, decided_by TEXT, result_branch TEXT);
CREATE TABLE IF NOT EXISTS ai_log (id INTEGER PRIMARY KEY AUTOINCREMENT, at TEXT NOT NULL, kind TEXT NOT NULL, model TEXT NOT NULL, input_tokens INTEGER NOT NULL, output_tokens INTEGER NOT NULL, cost_micro_usd INTEGER NOT NULL, actor TEXT NOT NULL, status TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS agent_profiles (name TEXT PRIMARY KEY, body TEXT NOT NULL, builtin INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS rejections (id INTEGER PRIMARY KEY AUTOINCREMENT, at TEXT NOT NULL, branch_id TEXT NOT NULL, event_type TEXT NOT NULL, code TEXT NOT NULL, message TEXT NOT NULL, principal TEXT NOT NULL);
CREATE TRIGGER IF NOT EXISTS events_no_update BEFORE UPDATE ON events BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;
CREATE TRIGGER IF NOT EXISTS events_no_delete BEFORE DELETE ON events WHEN (SELECT value FROM settings WHERE key = 'allow_world_delete') IS NOT '1' BEGIN SELECT RAISE(ABORT, 'events are immutable'); END;
"#;

impl Engine {
    pub fn open(path: &str) -> Res<Engine> {
        let conn = db(if path == ":memory:" { Connection::open_in_memory() } else { Connection::open(path) })?;
        db(conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;"))?;
        db(conn.execute_batch(SCHEMA))?;
        let mut e = Engine { conn, cache: HashMap::new(), clock: Box::new(now_iso) };
        crate::agents::install_builtin(&mut e)?;
        Ok(e)
    }
    /// Tests pin the wall clock so record hashes are reproducible.
    pub fn set_clock(&mut self, f: Box<dyn Fn() -> String + Send>) {
        self.clock = f;
    }
    pub fn now(&self) -> String {
        (self.clock)()
    }

    // ---- worlds & branches ---------------------------------------------------------
    pub fn create_world(&mut self, name: &str, seed: u64) -> Res<Branch> {
        let name = name.trim();
        if name.is_empty() || name.len() > 80 {
            return reject("invalid_payload", "world name must be 1..80 characters");
        }
        let n: i64 = db(self.conn.query_row("SELECT COUNT(*) FROM worlds", [], |r| r.get(0)))?;
        let world_id = format!("w{:03}_{}", n + 1, &sha(format!("{name}{seed}{}", self.now()).as_bytes())[..6]);
        let branch_id = format!("{world_id}.canon");
        let now = self.now();
        db(self.conn.execute("INSERT INTO worlds VALUES (?,?,?,?,?)", params![world_id, name, seed.to_string(), now, branch_id]))?;
        db(self.conn.execute("INSERT INTO branches VALUES (?,?,?,NULL,0,0,?,?,?)", params![branch_id, world_id, "Canon", "genesis", "canon", now]))?;
        self.cache.insert(branch_id.clone(), (0, WorldState::default()));
        self.commit(&branch_id, vec![Draft::new("world.genesis", json!({"world_id": world_id, "name": name, "seed": seed})).origin("system")], &Actor::owner(), None, None)?;
        self.branch(&branch_id)
    }
    pub fn worlds(&self) -> Res<Vec<Value>> {
        let mut st = db(self.conn.prepare("SELECT w.world_id, w.name, w.seed, w.created_at, w.canon_branch, (SELECT COUNT(*) FROM branches b WHERE b.world_id = w.world_id), (SELECT COUNT(*) FROM events e WHERE e.world_id = w.world_id) FROM worlds w ORDER BY w.created_at"))?;
        let rows = db(st.query_map([], |r| Ok(json!({"world_id": r.get::<_, String>(0)?, "name": r.get::<_, String>(1)?, "seed": r.get::<_, String>(2)?, "created_at": r.get::<_, String>(3)?, "canon_branch": r.get::<_, String>(4)?, "branches": r.get::<_, i64>(5)?, "events": r.get::<_, i64>(6)?}))))?;
        db(rows.collect())
    }
    pub fn delete_world(&mut self, world_id: &str) -> Res<()> {
        let tx = db(self.conn.transaction())?;
        db(tx.execute("INSERT OR REPLACE INTO settings VALUES ('allow_world_delete','1')", []))?;
        for t in ["event_causes WHERE event_id IN (SELECT event_id FROM events WHERE world_id = ?1)", "idempotency WHERE branch_id IN (SELECT branch_id FROM branches WHERE world_id = ?1)", "snapshots WHERE branch_id IN (SELECT branch_id FROM branches WHERE world_id = ?1)", "events WHERE world_id = ?1", "proposals WHERE world_id = ?1", "branches WHERE world_id = ?1", "worlds WHERE world_id = ?1"] {
            db(tx.execute("PRAGMA defer_foreign_keys=ON", []).and_then(|_| tx.execute(&format!("DELETE FROM {t}"), params![world_id])))?;
        }
        db(tx.execute("DELETE FROM settings WHERE key = 'allow_world_delete'", []))?;
        db(tx.commit())?;
        self.cache.retain(|k, _| !k.starts_with(&format!("{world_id}.")));
        Ok(())
    }
    pub fn branch(&self, id: &str) -> Res<Branch> {
        let b = db(self.conn.query_row("SELECT branch_id, world_id, name, parent_id, fork_seq, head_seq, head_hash, kind, created_at FROM branches WHERE branch_id = ?", params![id], |r| {
            Ok(Branch { branch_id: r.get(0)?, world_id: r.get(1)?, name: r.get(2)?, parent_id: r.get(3)?, fork_seq: r.get(4)?, head_seq: r.get(5)?, head_hash: r.get(6)?, kind: r.get(7)?, created_at: r.get(8)? })
        }).optional())?;
        b.ok_or(Reject { code: "unknown_branch".into(), message: format!("no branch '{id}'") })
    }
    pub fn branches(&self, world_id: &str) -> Res<Vec<Branch>> {
        let mut st = db(self.conn.prepare("SELECT branch_id FROM branches WHERE world_id = ? ORDER BY created_at, branch_id"))?;
        let ids: Vec<String> = db(db(st.query_map(params![world_id], |r| r.get(0)))?.collect())?;
        ids.iter().map(|i| self.branch(i)).collect()
    }
    pub fn canon_branch(&self, world_id: &str) -> Res<String> {
        db(self.conn.query_row("SELECT canon_branch FROM worlds WHERE world_id = ?", params![world_id], |r| r.get(0)).optional())?.ok_or(Reject { code: "unknown_world".into(), message: format!("no world '{world_id}'") })
    }
    /// Move the canonical pointer. Never deletes alternative histories.
    pub fn set_canon(&mut self, world_id: &str, branch_id: &str, actor: &Actor) -> Res<()> {
        if actor.role != "world_owner" {
            return reject("forbidden", "only the World Owner can move the canonical pointer");
        }
        if self.branch(branch_id)?.world_id != world_id {
            return reject("invalid_payload", "branch belongs to another world");
        }
        db(self.conn.execute("UPDATE worlds SET canon_branch = ? WHERE world_id = ?", params![branch_id, world_id]))?;
        Ok(())
    }

    /// Fork a new branch from the head of `parent`. The parent is not modified.
    pub fn fork(&mut self, parent: &str, name: &str, kind: &str, actor: &Actor) -> Res<Branch> {
        if !["world_owner", "gm"].contains(&actor.role.as_str()) {
            return reject("forbidden", "only the World Owner or a GM can create branches");
        }
        let p = self.branch(parent)?;
        let state = self.state(parent)?.clone();
        let n: i64 = db(self.conn.query_row("SELECT COUNT(*) FROM branches WHERE world_id = ?", params![p.world_id], |r| r.get(0)))?;
        let id = format!("{}.b{:03}", p.world_id, n);
        let h = state_hash(&state);
        let now = self.now();
        let tx = db(self.conn.transaction())?;
        db(tx.execute("INSERT INTO branches VALUES (?,?,?,?,?,0,?,?,?)", params![id, p.world_id, name.trim(), parent, p.head_seq, p.head_hash, kind, now]))?;
        db(tx.execute("INSERT INTO snapshots VALUES (?,0,?,?)", params![id, h, serde_json::to_string(&state).unwrap()]))?;
        db(tx.commit())?;
        self.cache.insert(id.clone(), (0, state));
        self.branch(&id)
    }

    // ---- state ---------------------------------------------------------------------
    fn row_to_event(r: &rusqlite::Row) -> rusqlite::Result<EventRecord> {
        let parse = |s: String| serde_json::from_str::<Value>(&s).unwrap_or(Value::Null);
        Ok(EventRecord {
            event_id: r.get(0)?, world_id: r.get(1)?, branch_id: r.get(2)?, seq: r.get(3)?, event_type: r.get(4)?, event_version: r.get(5)?,
            occurred_day: r.get(6)?, committed_at: r.get(7)?, causes: serde_json::from_value(parse(r.get(8)?)).unwrap_or_default(),
            actor: serde_json::from_value(parse(r.get(9)?)).unwrap_or(Actor::system()), origin: r.get(10)?, payload: parse(r.get(11)?),
            privacy: r.get(12)?, summary: r.get(13)?, prev_hash: r.get(14)?, record_hash: r.get(15)?, state_hash: r.get(16)?, derived: r.get::<_, i64>(17)? != 0,
        })
    }
    const EV_COLS: &'static str = "event_id, world_id, branch_id, seq, event_type, event_version, occurred_day, committed_at, causes_json, actor_json, origin, payload_json, privacy, summary, prev_hash, record_hash, state_hash, derived";

    pub fn events_range(&self, branch: &str, from_seq: i64, to_seq: i64) -> Res<Vec<EventRecord>> {
        let mut st = db(self.conn.prepare(&format!("SELECT {} FROM events WHERE branch_id = ? AND seq > ? AND seq <= ? ORDER BY seq", Self::EV_COLS)))?;
        let rows = db(st.query_map(params![branch, from_seq, to_seq], Self::row_to_event))?;
        db(rows.collect())
    }
    pub fn event(&self, id: &str) -> Res<EventRecord> {
        db(self.conn.query_row(&format!("SELECT {} FROM events WHERE event_id = ?", Self::EV_COLS), params![id], Self::row_to_event).optional())?.ok_or(Reject { code: "unknown_event".into(), message: format!("no event '{id}'") })
    }

    /// Materialised state at the head of a branch (cached; rebuilt from the nearest snapshot).
    pub fn state(&mut self, branch: &str) -> Res<&WorldState> {
        let b = self.branch(branch)?;
        let fresh = self.cache.get(branch).map(|(seq, _)| *seq == b.head_seq).unwrap_or(false);
        if !fresh {
            let snap: Option<(i64, String)> = db(self.conn.query_row("SELECT seq, state_json FROM snapshots WHERE branch_id = ? AND seq <= ? ORDER BY seq DESC LIMIT 1", params![branch, b.head_seq], |r| Ok((r.get(0)?, r.get(1)?))).optional())?;
            let (mut seq, mut state) = match snap {
                Some((q, js)) => (q, serde_json::from_str::<WorldState>(&js).map_err(|e| Reject { code: "corrupt_snapshot".into(), message: e.to_string() })?),
                None => (0, WorldState::default()),
            };
            for ev in self.events_range(branch, seq, b.head_seq)? {
                if !ev.derived {
                    reducer::apply(&mut state, &ev.event_type, &ev.payload)?;
                }
                seq = ev.seq;
            }
            self.cache.insert(branch.to_string(), (seq, state));
        }
        Ok(&self.cache.get(branch).unwrap().1)
    }

    // ---- commit --------------------------------------------------------------------
    /// Validate and atomically append events. Either every draft (and the incidents
    /// it derives) is committed, or nothing is and the head is unchanged.
    pub fn commit(&mut self, branch: &str, drafts: Vec<Draft>, actor: &Actor, idem_key: Option<&str>, expected_head: Option<i64>) -> Res<Committed> {
        let r = self.commit_inner(branch, &drafts, actor, idem_key, expected_head);
        if let Err(e) = &r {
            // Rejected commands leave a diagnostic record, never a canonical event.
            let et = drafts.first().map(|d| d.event_type.clone()).unwrap_or_default();
            let _ = self.conn.execute("INSERT INTO rejections (at, branch_id, event_type, code, message, principal) VALUES (?,?,?,?,?,?)", params![self.now(), branch, et, e.code, e.message, actor.principal]);
        }
        r
    }

    fn commit_inner(&mut self, branch: &str, drafts: &[Draft], actor: &Actor, idem_key: Option<&str>, expected_head: Option<i64>) -> Res<Committed> {
        let b = self.branch(branch)?;
        let req_hash = sha(serde_json::to_string(&drafts.iter().map(|d| json!([d.event_type, d.payload])).collect::<Vec<_>>()).unwrap().as_bytes());
        if let Some(k) = idem_key {
            let prior: Option<(String, String)> = db(self.conn.query_row("SELECT request_hash, response_json FROM idempotency WHERE branch_id = ? AND key = ?", params![branch, k], |r| Ok((r.get(0)?, r.get(1)?))).optional())?;
            if let Some((h, resp)) = prior {
                if h != req_hash {
                    return reject("idempotency_conflict", "this idempotency key was used for a different request");
                }
                let ids: Vec<String> = serde_json::from_str(&resp).unwrap_or_default();
                let events = ids.iter().map(|i| self.event(i)).collect::<Res<Vec<_>>>()?;
                return Ok(Committed { head_seq: b.head_seq, events, replayed: true });
            }
        }
        if let Some(h) = expected_head {
            if h != b.head_seq {
                return reject("stale_head", format!("expected head {h} but branch is at {}", b.head_seq));
            }
        }
        for d in drafts {
            if d.event_type.starts_with("chronicle.") {
                return reject("forbidden", "chronicle events are derived by the engine and cannot be submitted");
            }
            if !reducer::allowed_roles(&d.event_type).contains(&actor.role.as_str()) {
                return reject("forbidden", format!("role '{}' may not originate '{}'", actor.role, d.event_type));
            }
            for c in &d.causes {
                // causal references must point at events that exist in this world (ordinary DAG: causes precede effects)
                let w: Option<String> = db(self.conn.query_row("SELECT world_id FROM events WHERE event_id = ?", params![c], |r| r.get(0)).optional())?;
                if w.as_deref() != Some(b.world_id.as_str()) {
                    return reject("invalid_cause", format!("cause '{c}' is not an event of this world"));
                }
            }
        }
        let mut state = self.state(branch)?.clone();
        let now = self.now();
        let mut seq = b.head_seq;
        let mut prev = b.head_hash.clone();
        let mut records: Vec<EventRecord> = vec![];
        for d in drafts {
            let incidents = reducer::apply(&mut state, &d.event_type, &d.payload)?;
            let residual = state.conservation_residual();
            if let Some((c, v)) = residual.iter().find(|(_, v)| **v != 0) {
                return reject("invariant_violation", format!("conservation of {c} violated by {v} milli-units"));
            }
            if let Some(a) = state.negative_accounts().first() {
                return reject("invariant_violation", format!("account {a} would go negative"));
            }
            let m = &state.magic;
            if m.ambient < 0 || m.ambient + m.bound + m.dissipated != m.capacity_total {
                return reject("invariant_violation", "magical potential is not conserved");
            }
            let sh = state_hash(&state);
            seq += 1;
            let mut rec = EventRecord {
                event_id: format!("{branch}:{seq}"), world_id: b.world_id.clone(), branch_id: branch.to_string(), seq, event_type: d.event_type.clone(), event_version: reducer::EVENT_VERSION,
                occurred_day: state.meta.day, committed_at: now.clone(), causes: d.causes.clone(), actor: actor.clone(), origin: d.origin.clone(), payload: d.payload.clone(),
                privacy: reducer::privacy_of(&d.event_type, &d.payload).into(), summary: reducer::summarize(&state, &d.event_type, &d.payload), prev_hash: prev.clone(), record_hash: String::new(), state_hash: sh.clone(), derived: false,
            };
            rec.record_hash = rec.compute_hash();
            prev = rec.record_hash.clone();
            let main_id = rec.event_id.clone();
            records.push(rec);
            for i in incidents {
                seq += 1;
                let mut rec = EventRecord {
                    event_id: format!("{branch}:{seq}"), world_id: b.world_id.clone(), branch_id: branch.to_string(), seq, event_type: format!("chronicle.{}", i.kind), event_version: reducer::EVENT_VERSION,
                    occurred_day: state.meta.day, committed_at: now.clone(), causes: vec![main_id.clone()], actor: Actor::system(), origin: "simulation".into(), payload: i.data,
                    privacy: if i.public { "public" } else { "gm_private" }.into(), summary: i.text, prev_hash: prev.clone(), record_hash: String::new(), state_hash: sh.clone(), derived: true,
                };
                rec.record_hash = rec.compute_hash();
                prev = rec.record_hash.clone();
                records.push(rec);
            }
        }
        let snapshot = seq / SNAPSHOT_EVERY != b.head_seq / SNAPSHOT_EVERY;
        let tx = db(self.conn.transaction())?;
        {
            let mut ins = db(tx.prepare("INSERT INTO events VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"))?;
            let mut cause = db(tx.prepare("INSERT OR IGNORE INTO event_causes VALUES (?,?,?)"))?;
            for r in &records {
                db(ins.execute(params![r.event_id, r.world_id, r.branch_id, r.seq, r.event_type, r.event_version, r.occurred_day, r.committed_at, serde_json::to_string(&r.causes).unwrap(), serde_json::to_string(&r.actor).unwrap(), r.origin, serde_json::to_string(&r.payload).unwrap(), r.privacy, r.summary, r.prev_hash, r.record_hash, r.state_hash, r.derived as i64]))?;
                for c in &r.causes {
                    db(cause.execute(params![r.event_id, c, if r.derived { "produced" } else { "causes" }]))?;
                }
            }
        }
        // optimistic concurrency at the storage level too: the head must still be where we read it
        let n = db(tx.execute("UPDATE branches SET head_seq = ?, head_hash = ? WHERE branch_id = ? AND head_seq = ?", params![seq, prev, branch, b.head_seq]))?;
        if n != 1 {
            return reject("stale_head", "branch head moved during commit");
        }
        if snapshot {
            db(tx.execute("INSERT OR REPLACE INTO snapshots VALUES (?,?,?,?)", params![branch, seq, state_hash(&state), serde_json::to_string(&state).unwrap()]))?;
            db(tx.execute("DELETE FROM snapshots WHERE branch_id = ?1 AND seq > 0 AND seq NOT IN (SELECT seq FROM snapshots WHERE branch_id = ?1 ORDER BY seq DESC LIMIT 2)", params![branch]))?;
        }
        if let Some(k) = idem_key {
            let ids: Vec<&String> = records.iter().map(|r| &r.event_id).collect();
            db(tx.execute("INSERT INTO idempotency VALUES (?,?,?,?)", params![branch, k, req_hash, serde_json::to_string(&ids).unwrap()]))?;
        }
        db(tx.commit())?;
        self.cache.insert(branch.to_string(), (seq, state));
        Ok(Committed { head_seq: seq, events: records, replayed: false })
    }

    /// Dry-run: what would these drafts do? Nothing is written.
    pub fn preview(&mut self, branch: &str, drafts: &[Draft]) -> Res<(WorldState, Vec<String>)> {
        let mut state = self.state(branch)?.clone();
        let mut notes = vec![];
        for d in drafts {
            for i in reducer::apply(&mut state, &d.event_type, &d.payload)? {
                notes.push(i.text);
            }
        }
        Ok((state, notes))
    }

    // ---- lineage, replay & verification ----------------------------------------------
    /// [(branch, upto_seq)] from the root down to `branch`.
    pub fn lineage(&self, branch: &str) -> Res<Vec<(String, i64)>> {
        let mut out = vec![];
        let mut cur = self.branch(branch)?;
        let mut upto = cur.head_seq;
        loop {
            out.push((cur.branch_id.clone(), upto));
            match cur.parent_id.clone() {
                Some(p) => {
                    upto = cur.fork_seq;
                    cur = self.branch(&p)?;
                }
                None => break,
            }
        }
        out.reverse();
        Ok(out)
    }

    /// Rebuild state purely from events (no snapshots, no cache), checking every hash on the way.
    pub fn verify(&mut self, branch: &str) -> Res<Value> {
        let lineage = self.lineage(branch)?;
        let mut state = WorldState::default();
        let mut problems: Vec<String> = vec![];
        let mut checked = 0;
        let mut prev = "genesis".to_string();
        for (bid, upto) in &lineage {
            for ev in self.events_range(bid, 0, *upto)? {
                checked += 1;
                if ev.prev_hash != prev {
                    problems.push(format!("{}: hash chain broken", ev.event_id));
                }
                if ev.compute_hash() != ev.record_hash {
                    problems.push(format!("{}: record hash mismatch (content altered)", ev.event_id));
                }
                prev = ev.record_hash.clone();
                if !ev.derived {
                    if let Err(e) = reducer::apply(&mut state, &ev.event_type, &ev.payload) {
                        problems.push(format!("{}: replay rejected: {e}", ev.event_id));
                        continue;
                    }
                }
                if state_hash(&state) != ev.state_hash {
                    problems.push(format!("{}: replayed state differs from recorded state hash", ev.event_id));
                }
            }
        }
        let replay_hash = state_hash(&state);
        let b = self.branch(branch)?;
        if b.head_hash != prev {
            problems.push("branch head hash does not match the last event".into());
        }
        let mut st = db(self.conn.prepare("SELECT seq, state_hash, state_json FROM snapshots WHERE branch_id = ?"))?;
        let snaps: Vec<(i64, String, String)> = db(db(st.query_map(params![branch], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))))?.collect())?;
        drop(st);
        for (seq, h, js) in snaps {
            if sha(js.as_bytes()) != h {
                let reser = serde_json::from_str::<WorldState>(&js).map(|s| state_hash(&s)).unwrap_or_default();
                if reser != h {
                    problems.push(format!("snapshot at seq {seq} does not match its recorded hash (tampered or corrupt)"));
                }
            }
        }
        // materialised (cached) projection must equal the replay
        self.cache.remove(branch);
        let cached = state_hash(self.state(branch)?);
        if cached != replay_hash {
            problems.push("materialised state differs from full replay".into());
        }
        Ok(json!({"ok": problems.is_empty(), "branch": branch, "events_checked": checked, "state_hash": replay_hash, "residual": state.conservation_residual(), "problems": problems}))
    }

    pub fn setting(&self, key: &str) -> Option<String> {
        self.conn.query_row("SELECT value FROM settings WHERE key = ?", params![key], |r| r.get(0)).optional().ok().flatten()
    }
    pub fn set_setting(&self, key: &str, value: &str) -> Res<()> {
        db(self.conn.execute("INSERT OR REPLACE INTO settings VALUES (?,?)", params![key, value]))?;
        Ok(())
    }
}
