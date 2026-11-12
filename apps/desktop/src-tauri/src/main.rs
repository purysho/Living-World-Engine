//! Desktop shell. The window is only a view: every command goes through one
//! IPC entry point into `lwe_core::api::App::dispatch` — the same gateway the
//! HTTP host uses — so nothing in the web layer can write world state directly.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lwe_core::api::App;
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::Manager;

struct Engine(Mutex<App>);

const KEY_SERVICE: &str = "LivingWorldEngine";
const KEY_USER: &str = "anthropic_api_key";

/// API keys live in the Windows Credential Manager, never in a world file.
#[cfg(windows)]
fn load_key() -> Option<String> {
    keyring::Entry::new(KEY_SERVICE, KEY_USER).ok()?.get_password().ok().filter(|k| !k.is_empty())
}
#[cfg(windows)]
fn store_key(key: &str) {
    if let Ok(e) = keyring::Entry::new(KEY_SERVICE, KEY_USER) {
        if key.trim().is_empty() {
            let _ = e.delete_credential();
        } else {
            let _ = e.set_password(key.trim());
        }
    }
}
#[cfg(not(windows))]
fn load_key() -> Option<String> {
    let _ = (KEY_SERVICE, KEY_USER);
    None
}
#[cfg(not(windows))]
fn store_key(_key: &str) {}

#[tauri::command(async)]
fn lwe_dispatch(state: tauri::State<'_, Engine>, request: Value) -> Value {
    if request["op"] == "ai.config.set" {
        if let Some(k) = request["args"]["api_key"].as_str() {
            store_key(k);
        }
    }
    match state.0.lock() {
        Ok(mut app) => app.dispatch(&request),
        Err(_) => json!({"ok": false, "error": {"code": "engine_poisoned", "message": "The engine stopped after an internal error. Restart the app; your worlds are safe on disk."}}),
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = dir.join("worlds.db");
            let mut engine = App::open(&db.to_string_lossy()).map_err(|e| format!("cannot open {}: {e}", db.display()))?;
            if engine.api_key.is_none() {
                engine.api_key = load_key();
            }
            app.manage(Engine(Mutex::new(engine)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![lwe_dispatch])
        .run(tauri::generate_context!())
        .expect("error while running Living World Engine");
}
