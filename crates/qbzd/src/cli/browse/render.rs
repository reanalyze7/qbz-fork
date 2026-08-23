use serde_json::Value;

/// Generic human list: every object carried in an `items`/`tracks` array,
/// rendered as `id  Artist — Title`. Robust across the album/artist/similar/
/// suggest payload shapes (the `--json` output is the exact contract). Shared
/// with `cli::fav` (favorites payloads are the same items-array shape).
pub(crate) fn render(p: &Value) -> String {
    let mut items: Vec<&Value> = Vec::new();
    walk(p, &mut items);
    if items.is_empty() {
        return "no results\n".to_string();
    }
    let mut out = String::new();
    for it in items {
        let id = id_str(it.get("id"));
        if id.is_empty() {
            continue;
        }
        let title = it
            .get("title")
            .or_else(|| it.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("(untitled)");
        match secondary(it) {
            Some(s) => out.push_str(&format!("{id}  {s} — {title}\n")),
            None => out.push_str(&format!("{id}  {title}\n")),
        }
    }
    if out.is_empty() {
        out.push_str("no results\n");
    }
    out
}

pub(crate) fn collect_ids(p: &Value) -> Vec<String> {
    let mut items: Vec<&Value> = Vec::new();
    walk(p, &mut items);
    items
        .iter()
        .map(|it| id_str(it.get("id")))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Collect objects held in `items`/`tracks` arrays anywhere in the payload.
/// Nested reference objects (a track's `artist`/`album`) are NOT under those
/// keys, so they are not over-collected.
fn walk<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                if (k == "items" || k == "tracks") && val.is_array() {
                    if let Value::Array(arr) = val {
                        for e in arr {
                            if e.is_object() && e.get("id").is_some() {
                                out.push(e);
                            }
                        }
                    }
                }
                walk(val, out);
            }
        }
        Value::Array(arr) => {
            for e in arr {
                walk(e, out);
            }
        }
        _ => {}
    }
}

fn secondary(it: &Value) -> Option<&str> {
    it.get("artist")
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .or_else(|| it.get("performer").and_then(|a| a.get("name")).and_then(|v| v.as_str()))
}

fn id_str(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}
