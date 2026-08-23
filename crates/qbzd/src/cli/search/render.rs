use serde_json::Value;

/// The category order used by both the human table and `--ids`. `tracks` leads
/// so `qbzd search "..." --type tracks --ids | qbzd queue add -` is the obvious
/// pipeline; the order is stable so scripts can rely on it.
pub(super) const CATEGORIES: [(&str, &str); 4] = [
    ("tracks", "TRACKS"),
    ("albums", "ALBUMS"),
    ("artists", "ARTISTS"),
    ("playlists", "PLAYLISTS"),
];

/// Human top-hits table. Defensive against missing optional fields — a category
/// with no results is skipped; an item with no title/artist degrades gracefully
/// rather than erroring (the `--json` payload is the exact contract; this is a
/// convenience view).
pub(super) fn render(p: &Value) -> String {
    let mut out = String::new();
    for (key, label) in CATEGORIES {
        let page = match p.get(key) {
            Some(v) if v.is_object() => v,
            _ => continue,
        };
        let items = match page.get("items").and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => a,
            _ => continue,
        };
        let total = page
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(items.len() as u64);
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("{label} ({total})\n"));
        for it in items {
            let id = id_str(it.get("id"));
            let title = it
                .get("title")
                .or_else(|| it.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("(untitled)");
            match secondary_name(it) {
                Some(s) => out.push_str(&format!("  {id}  {s} — {title}\n")),
                None => out.push_str(&format!("  {id}  {title}\n")),
            }
        }
    }
    if out.is_empty() {
        out.push_str("no results\n");
    }
    out
}

/// The ids of every returned item, in `CATEGORIES` order — the composition
/// currency. For a typed search only one category is populated; for `--type
/// all` this emits all ids (album ids are strings, track/artist ids numbers —
/// realistic pipelines use a typed search, but mixing is not an error).
pub(super) fn collect_ids(p: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    for (key, _label) in CATEGORIES {
        if let Some(items) = p
            .get(key)
            .and_then(|v| v.get("items"))
            .and_then(|v| v.as_array())
        {
            for it in items {
                let id = id_str(it.get("id"));
                if !id.is_empty() {
                    ids.push(id);
                }
            }
        }
    }
    ids
}

/// A best-effort "artist" line: an album/track carries `artist.name` or
/// `performer.name`; an artist/playlist has neither (its `name` is the title).
fn secondary_name(it: &Value) -> Option<&str> {
    it.get("artist")
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            it.get("performer")
                .and_then(|a| a.get("name"))
                .and_then(|v| v.as_str())
        })
}

/// Stringify an id `Value` without quotes: a string id (album) prints bare, a
/// numeric id (track/artist/playlist) prints as its integer. Missing/other →
/// empty (skipped by the callers).
pub(super) fn id_str(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}
