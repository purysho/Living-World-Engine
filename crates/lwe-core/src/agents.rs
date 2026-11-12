//! Agent profiles: the `.md` specifications that govern how an NPC is voiced.
//! Built-ins are compiled in from `/agents`; installations may edit copies.
//! Scope and permissions are derived from *tier in world state*, never from
//! anything a model or a profile file says about itself.

use crate::model::*;
use crate::store::Engine;
use rusqlite::params;
use serde_json::{json, Value};

pub const BUILTIN: [(&str, &str); 5] = [
    ("t1-resident", include_str!("../../../agents/generic/t1-resident.md")),
    ("t2-recurring", include_str!("../../../agents/generic/t2-recurring.md")),
    ("t3-major", include_str!("../../../agents/generic/t3-major.md")),
    ("t4-world-shaper", include_str!("../../../agents/generic/t4-world-shaper.md")),
    ("custom-schemer", include_str!("../../../agents/custom/custom-schemer.md")),
];

fn db<T>(r: rusqlite::Result<T>) -> Res<T> {
    r.map_err(|e| Reject { code: "storage".into(), message: e.to_string() })
}

pub fn install_builtin(e: &mut Engine) -> Res<()> {
    for (name, body) in BUILTIN {
        db(e.conn.execute("INSERT OR IGNORE INTO agent_profiles VALUES (?,?,1)", params![name, body]))?;
    }
    Ok(())
}

pub fn list(e: &Engine) -> Res<Vec<Value>> {
    let mut st = db(e.conn.prepare("SELECT name, body, builtin FROM agent_profiles ORDER BY name"))?;
    let rows = db(st.query_map([], |r| {
        let body: String = r.get(1)?;
        let name: String = r.get(0)?;
        let original = BUILTIN.iter().find(|(n, _)| *n == name).map(|(_, b)| *b);
        Ok(json!({"name": name, "builtin": r.get::<_, i64>(2)? != 0, "modified": original.map(|o| o != body).unwrap_or(false), "tier": tier_of(&body), "role": front(&body, "role"), "body": body}))
    }))?;
    db(rows.collect())
}

pub fn get(e: &Engine, name: &str) -> Res<String> {
    use rusqlite::OptionalExtension;
    let b: Option<String> = db(e.conn.query_row("SELECT body FROM agent_profiles WHERE name = ?", params![name], |r| r.get(0)).optional())?;
    match b {
        Some(b) => Ok(b),
        None => get(e, "t1-resident"),
    }
}

pub fn set(e: &Engine, name: &str, body: &str) -> Res<()> {
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || name.is_empty() || name.len() > 48 {
        return reject("invalid_payload", "profile names use letters, digits and hyphens");
    }
    if body.len() > 20_000 || !body.trim_start().starts_with("---") {
        return reject("invalid_payload", "a profile is Markdown with YAML front-matter, at most 20k characters");
    }
    db(e.conn.execute("INSERT INTO agent_profiles VALUES (?1,?2,0) ON CONFLICT(name) DO UPDATE SET body = ?2", params![name, body]))?;
    Ok(())
}

pub fn reset(e: &Engine, name: &str) -> Res<()> {
    match BUILTIN.iter().find(|(n, _)| *n == name) {
        Some((_, b)) => {
            db(e.conn.execute("UPDATE agent_profiles SET body = ? WHERE name = ?", params![b, name]))?;
            Ok(())
        }
        None => {
            db(e.conn.execute("DELETE FROM agent_profiles WHERE name = ? AND builtin = 0", params![name]))?;
            Ok(())
        }
    }
}

pub fn front(body: &str, key: &str) -> String {
    let mut parts = body.splitn(3, "---");
    parts.next();
    parts.next().unwrap_or("").lines().find_map(|l| l.strip_prefix(&format!("{key}:")).map(|v| v.trim().trim_matches('"').to_string())).unwrap_or_default()
}
fn tier_of(body: &str) -> i64 {
    front(body, "tier").parse().unwrap_or(1)
}
/// The persona instructions (everything after the front-matter).
pub fn instructions(body: &str) -> String {
    body.splitn(3, "---").nth(2).unwrap_or(body).trim().to_string()
}
