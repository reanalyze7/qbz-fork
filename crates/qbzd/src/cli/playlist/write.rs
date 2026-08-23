use serde_json::Value;

use crate::paths::ProfileRoots;

use super::internal::{post, resolve_ids};

/// `qbzd playlist create <NAME> [--desc D] [--public]`.
pub async fn create(host: Option<String>, name: String, desc: Option<String>, public: bool, roots: &ProfileRoots) -> i32 {
    let mut body = serde_json::json!({ "name": name, "public": public });
    if let Some(d) = desc {
        body["description"] = Value::String(d);
    }
    post(host, roots, "/api/playlist/create", body, |v| {
        let pl = v.get("playlist");
        let id = pl.and_then(|p| p.get("id")).and_then(|x| x.as_u64()).unwrap_or(0);
        let nm = pl.and_then(|p| p.get("name")).and_then(|x| x.as_str()).unwrap_or("");
        format!("created playlist {id} \"{nm}\"")
    })
    .await
}

/// `qbzd playlist edit <ID> [--name N] [--desc D] [--public|--private]`.
#[allow(clippy::too_many_arguments)]
pub async fn edit(host: Option<String>, id: u64, name: Option<String>, desc: Option<String>, public: bool, private: bool, roots: &ProfileRoots) -> i32 {
    if public && private {
        eprintln!("error: --public and --private are mutually exclusive");
        return 2;
    }
    let mut body = serde_json::json!({ "id": id });
    if let Some(n) = name {
        body["name"] = Value::String(n);
    }
    if let Some(d) = desc {
        body["description"] = Value::String(d);
    }
    if public {
        body["public"] = Value::Bool(true);
    } else if private {
        body["public"] = Value::Bool(false);
    }
    post(host, roots, "/api/playlist/update", body, |_| "playlist updated".to_string()).await
}

/// `qbzd playlist rm <ID> --yes`.
pub async fn rm(host: Option<String>, id: u64, yes: bool, roots: &ProfileRoots) -> i32 {
    if !yes {
        eprintln!("error: refusing to delete without --yes");
        eprintln!("  → qbzd playlist rm {id} --yes");
        return 2;
    }
    post(host, roots, "/api/playlist/delete", serde_json::json!({ "id": id }), move |_| {
        format!("deleted playlist {id}")
    })
    .await
}

/// `qbzd playlist add <ID> <TRACK_IDS...|->`.
pub async fn add(host: Option<String>, id: u64, track_ids: Vec<String>, roots: &ProfileRoots) -> i32 {
    let ids = match resolve_ids(track_ids) {
        Ok(i) => i,
        Err(m) => {
            eprintln!("error: {m}");
            return 2;
        }
    };
    let body = serde_json::json!({ "id": id, "track_ids": ids });
    post(host, roots, "/api/playlist/tracks/add", body, |v| {
        let n = v.get("added").and_then(|x| x.as_u64()).unwrap_or(0);
        format!("added {n} track(s)")
    })
    .await
}

/// `qbzd playlist remove <ID> <TRACK_IDS...>` (plain track ids; the daemon
/// resolves them to per-playlist row ids).
pub async fn remove(host: Option<String>, id: u64, track_ids: Vec<String>, roots: &ProfileRoots) -> i32 {
    let ids = match resolve_ids(track_ids) {
        Ok(i) => i,
        Err(m) => {
            eprintln!("error: {m}");
            return 2;
        }
    };
    let body = serde_json::json!({ "id": id, "track_ids": ids });
    post(host, roots, "/api/playlist/tracks/remove", body, |v| {
        let n = v.get("removed").and_then(|x| x.as_u64()).unwrap_or(0);
        format!("removed {n} track(s)")
    })
    .await
}
