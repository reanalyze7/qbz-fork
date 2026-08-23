// crates/qbzd/src/cli/browse/ — the catalog READ verbs (02 §2.3):
// `qbzd album`, `qbzd artist`, `qbzd similar`, `qbzd suggest`. Each is a
// stateless renderer over one GET request. Three modes on every verb: default
// human list, `--ids` (ids one-per-line — the composition currency), `--json`
// (the raw payload). The human/`--ids` views walk the payload generically
// (items/tracks arrays); `--json` is the exact, complete contract.
mod render;
#[cfg(test)]
mod tests;

use std::io::Read;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

pub(crate) use render::{collect_ids, render};

/// `qbzd album <ALBUM_ID> [--suggest] [--ids] [--json]`.
pub async fn album(host: Option<String>, id: String, suggest: bool, ids: bool, json: bool, roots: &ProfileRoots) -> i32 {
    let path = format!("/api/album?id={}&suggest={}", urlencoding::encode(&id), if suggest { 1 } else { 0 });
    get_and_render(host, roots, &path, ids, json).await
}

/// `qbzd artist <ARTIST_ID> [--top|--albums] [--limit N] [--ids] [--json]`.
#[allow(clippy::too_many_arguments)]
pub async fn artist(host: Option<String>, id: u64, top: bool, albums: bool, limit: u32, ids: bool, json: bool, roots: &ProfileRoots) -> i32 {
    let view = if albums { "albums" } else if top { "top" } else { "page" };
    let path = format!("/api/artist?id={id}&view={view}&limit={limit}");
    get_and_render(host, roots, &path, ids, json).await
}

/// `qbzd similar <artist:ID | album:ID> [--limit N] [--ids] [--json]`.
pub async fn similar(host: Option<String>, selector: String, limit: u32, ids: bool, json: bool, roots: &ProfileRoots) -> i32 {
    let path = match to_similar_query(&selector, limit) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!("  → usage: qbzd similar artist:<ID> | album:<ID>");
            return 2;
        }
    };
    get_and_render(host, roots, &path, ids, json).await
}

/// `qbzd suggest [--seed <ID,ID> | --seed -] [--limit N] [--ids] [--json]`.
/// No `--seed` = the daemon seeds from the current queue. `--seed -` reads ids
/// one-per-line from stdin.
pub async fn suggest(host: Option<String>, seed: Option<String>, limit: u32, ids: bool, json: bool, roots: &ProfileRoots) -> i32 {
    let seed_param = match seed.as_deref() {
        Some("-") => Some(read_stdin_ids()),
        Some(s) => Some(s.to_string()),
        None => None,
    };
    let mut path = format!("/api/suggest?limit={limit}");
    if let Some(s) = seed_param.filter(|s| !s.is_empty()) {
        path.push_str(&format!("&seed={}", urlencoding::encode(&s)));
    }
    get_and_render(host, roots, &path, ids, json).await
}

// ============================ shared ============================

async fn get_and_render(host: Option<String>, roots: &ProfileRoots, path: &str, ids: bool, json: bool) -> i32 {
    let client = ApiClient::new(host, roots);
    let payload = match client.get(path).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return e.exit_code();
        }
    };
    if json {
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else if ids {
        for id in collect_ids(&payload) {
            println!("{id}");
        }
    } else {
        print!("{}", render(&payload));
    }
    0
}

fn to_similar_query(selector: &str, limit: u32) -> Result<String, String> {
    let s = selector.trim();
    if let Some(id) = s.strip_prefix("artist:") {
        let id: u64 = id.parse().map_err(|_| format!("'{id}' is not a numeric artist id"))?;
        return Ok(format!("/api/similar?artist={id}&limit={limit}"));
    }
    if let Some(id) = s.strip_prefix("album:") {
        if id.is_empty() {
            return Err("album id is empty".into());
        }
        return Ok(format!("/api/similar?album={}&limit={limit}", urlencoding::encode(id)));
    }
    Err(format!("unrecognized selector '{s}'"))
}

fn read_stdin_ids() -> String {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    buf.split_whitespace().collect::<Vec<_>>().join(",")
}
