//! Minimal local host: `POST /api` → `App::dispatch`, plus static files for the UI.
//! Used for development, automated UI tests and headless play. It binds to
//! loopback only and has no authentication: do not expose it to a network.
//! (A hosted multi-user server is a later milestone with its own security gate.)

use lwe_core::api::App;
use std::io::Read;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Response, Server};

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned().unwrap_or_else(|| default.to_string())
}

fn mime(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" => "application/json",
        "png" => "image/png",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn main() {
    let db = arg("--db", "worlds.db");
    let ui = PathBuf::from(arg("--ui", "apps/desktop/dist"));
    let port = arg("--port", "4747");
    let mut app = App::open(&db).unwrap_or_else(|e| {
        eprintln!("cannot open {db}: {e}");
        std::process::exit(1)
    });
    let server = Server::http(format!("127.0.0.1:{port}")).unwrap_or_else(|e| {
        eprintln!("cannot bind 127.0.0.1:{port}: {e}");
        std::process::exit(1)
    });
    println!("Living World Engine {} listening on http://127.0.0.1:{port} (db: {db}, ui: {})", lwe_core::APP_VERSION, ui.display());
    let json_h = Header::from_bytes("Content-Type", "application/json").unwrap();
    for mut req in server.incoming_requests() {
        let url = req.url().split('?').next().unwrap_or("/").to_string();
        if req.method() == &Method::Post && url == "/api" {
            // same-origin only: refuse cross-site browser requests to the local gateway
            let origin_ok = req.headers().iter().find(|h| h.field.equiv("Origin")).map(|h| h.value.as_str().starts_with(&format!("http://127.0.0.1:{port}")) || h.value.as_str().starts_with(&format!("http://localhost:{port}"))).unwrap_or(true);
            let mut body = String::new();
            let ok = origin_ok && req.as_reader().take(4_000_000).read_to_string(&mut body).is_ok();
            let out = match (ok, serde_json::from_str::<serde_json::Value>(&body)) {
                (true, Ok(v)) => app.dispatch(&v),
                (false, _) => serde_json::json!({"ok": false, "error": {"code": "forbidden", "message": "cross-origin or unreadable request"}}),
                (_, Err(e)) => serde_json::json!({"ok": false, "error": {"code": "invalid_request", "message": e.to_string()}}),
            };
            let _ = req.respond(Response::from_string(out.to_string()).with_header(json_h.clone()));
            continue;
        }
        let rel = url.trim_start_matches('/');
        let mut path = ui.join(if rel.is_empty() { "index.html" } else { rel });
        // never serve outside the UI directory
        if rel.contains("..") || !path.starts_with(&ui) || !path.is_file() {
            path = ui.join("index.html");
        }
        match std::fs::read(&path) {
            Ok(bytes) => {
                let _ = req.respond(Response::from_data(bytes).with_header(Header::from_bytes("Content-Type", mime(&path)).unwrap()));
            }
            Err(_) => {
                let _ = req.respond(Response::from_string("UI not built. Run `npm run build` in apps/desktop.").with_status_code(404));
            }
        }
    }
}
