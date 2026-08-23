use serde_json::Value;

use super::render::{collect_ids, id_str, render};

fn payload() -> Value {
    serde_json::json!({
        "query": "feather", "type": "all", "limit": 20, "offset": 0,
        "albums": {"items": [
            {"id": "c9vd8vvvrbpkc", "title": "Light as a Feather",
             "artist": {"name": "Chick Corea"}}
        ], "total": 27, "offset": 0, "limit": 20},
        "tracks": {"items": [
            {"id": 176544871, "title": "Spain", "performer": {"name": "Chick Corea"}}
        ], "total": 5, "offset": 0, "limit": 20},
        "artists": Value::Null,
        "playlists": Value::Null
    })
}

#[test]
fn collect_ids_leads_with_tracks_then_albums() {
    // CATEGORIES order: tracks first (queue-pipe target), then albums.
    assert_eq!(collect_ids(&payload()), vec!["176544871", "c9vd8vvvrbpkc"]);
}

#[test]
fn id_str_handles_string_and_numeric_ids_without_quotes() {
    assert_eq!(
        id_str(Some(&serde_json::json!("c9vd8vvvrbpkc"))),
        "c9vd8vvvrbpkc"
    );
    assert_eq!(id_str(Some(&serde_json::json!(176544871u64))), "176544871");
    assert_eq!(id_str(Some(&Value::Null)), "");
    assert_eq!(id_str(None), "");
}

#[test]
fn render_shows_present_categories_with_ids_and_names() {
    let out = render(&payload());
    assert!(out.contains("TRACKS (5)"), "{out}");
    assert!(out.contains("  176544871  Chick Corea — Spain"), "{out}");
    assert!(out.contains("ALBUMS (27)"), "{out}");
    assert!(
        out.contains("  c9vd8vvvrbpkc  Chick Corea — Light as a Feather"),
        "{out}"
    );
    // null categories are skipped entirely.
    assert!(!out.contains("ARTISTS"), "{out}");
    assert!(!out.contains("PLAYLISTS"), "{out}");
}

#[test]
fn render_empty_payload_says_no_results() {
    let empty = serde_json::json!({"albums": Value::Null, "tracks": Value::Null});
    assert_eq!(render(&empty), "no results\n");
}
