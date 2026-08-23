use crate::cli::queue::fmt::fmt_mmss;
use crate::cli::queue::mutate::{render_added, render_removed};

// ------------------------------ add rendering ------------------------------

#[test]
fn render_added_uses_the_materialized_track_and_next_suffix() {
    let v = serde_json::json!({
        "added": 1, "total_tracks": 15,
        "tracks": [{"id": 176544872, "title": "Spain", "artist": "Chick Corea"}]
    });
    assert_eq!(render_added(&v, true), "added: Spain – Chick Corea (next)");
    assert_eq!(render_added(&v, false), "added: Spain – Chick Corea");
}

#[test]
fn render_added_falls_back_to_the_bare_count_without_tracks() {
    let v = serde_json::json!({"added": 1, "total_tracks": 15});
    assert_eq!(render_added(&v, false), "added: 1 track(s)");
}

// ---------------------------- remove rendering ----------------------------

#[test]
fn render_removed_shows_id_and_remaining_count() {
    let v = serde_json::json!({"removed": 176544872, "total_tracks": 14});
    assert_eq!(render_removed(&v), "removed: track 176544872 · 14 left");
}

#[test]
fn fmt_mmss_pads_seconds() {
    assert_eq!(fmt_mmss(581), "9:41");
    assert_eq!(fmt_mmss(65), "1:05");
}
