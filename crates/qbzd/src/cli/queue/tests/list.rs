use crate::cli::queue::list::{render_queue_list, HEADER};

// -------------------------- queue list rendering --------------------------

#[test]
fn render_queue_list_reproduces_the_documented_example_byte_exact() {
    // 02 §2.2's documented state, verbatim: current_index=1 (0-based) ->
    // "Spain" at row #2 with the `->` marker; the played "Captain
    // Marvel" (from the additive `history` field, recent-first) at row
    // #1 above it; "500 Miles High" (the sole `upcoming` entry) at #3.
    // Every byte, including column padding, is the spec's.
    let v = serde_json::json!({
        "current_track": {"id": 176544871, "title": "Spain", "artist": "Chick Corea", "duration_secs": 581},
        "current_index": 1,
        "upcoming": [
            {"id": 176544872, "title": "500 Miles High", "artist": "Chick Corea", "duration_secs": 547}
        ],
        "history": [
            {"id": 176544870, "title": "Captain Marvel", "artist": "Chick Corea", "duration_secs": 293}
        ],
        "history_len": 1, "shuffle": false, "repeat": "off",
        "total_tracks": 14, "stop_after_track_id": null, "offset": 0, "limit": 100
    });
    let expected = concat!(
        "    #  track                                   artist            len\n",
        "    1  Captain Marvel                          Chick Corea      4:53\n",
        "->  2  Spain                                   Chick Corea      9:41\n",
        "    3  500 Miles High                          Chick Corea      9:07\n",
        "14 tracks · shuffle off · repeat off\n",
    );
    assert_eq!(render_queue_list(&v), expected);
}

#[test]
fn render_queue_list_caps_history_rows_and_never_goes_below_position_one() {
    // 5 history entries, current at position 6 -> only the 3 most recent
    // render (positions 3, 4, 5 — oldest of the three first).
    let history: Vec<_> = (0..5)
        .map(|i| serde_json::json!({"id": i, "title": format!("H{i}"), "artist": "X", "duration_secs": 60}))
        .collect();
    let v = serde_json::json!({
        "current_track": {"id": 99, "title": "Cur", "artist": "X", "duration_secs": 60},
        "current_index": 5,
        "upcoming": [],
        "history": history,
        "history_len": 5, "shuffle": false, "repeat": "off",
        "total_tracks": 6, "stop_after_track_id": null, "offset": 0, "limit": 100
    });
    let rendered = render_queue_list(&v);
    // history is recent-first: H0 is the most recent -> directly above
    // the current row at position 5; H2 the oldest rendered at 3.
    assert!(rendered.contains("    3  H2"), "{rendered}");
    assert!(rendered.contains("    4  H1"), "{rendered}");
    assert!(rendered.contains("    5  H0"), "{rendered}");
    assert!(!rendered.contains("H3"), "{rendered}");
    assert!(!rendered.contains("H4"), "{rendered}");
    assert!(rendered.contains("->  6  Cur"), "{rendered}");

    // Current at position 2 with 3 history entries -> only 1 row fits
    // above it (positions never go below 1).
    let clipped = serde_json::json!({
        "current_track": {"id": 99, "title": "Cur", "artist": "X", "duration_secs": 60},
        "current_index": 1,
        "upcoming": [],
        "history": [
            {"id": 0, "title": "H0", "artist": "X", "duration_secs": 60},
            {"id": 1, "title": "H1", "artist": "X", "duration_secs": 60},
            {"id": 2, "title": "H2", "artist": "X", "duration_secs": 60}
        ],
        "history_len": 3, "shuffle": false, "repeat": "off",
        "total_tracks": 2, "stop_after_track_id": null, "offset": 0, "limit": 100
    });
    let rendered = render_queue_list(&clipped);
    assert!(rendered.contains("    1  H0"), "{rendered}");
    assert!(!rendered.contains("H1"), "{rendered}");
    assert!(rendered.contains("->  2  Cur"), "{rendered}");
}

#[test]
fn render_queue_list_numbers_from_one_when_nothing_is_current() {
    let v = serde_json::json!({
        "current_track": null, "current_index": null,
        "upcoming": [
            {"id": 1, "title": "A", "artist": "X", "duration_secs": 60},
            {"id": 2, "title": "B", "artist": "Y", "duration_secs": 60}
        ],
        "history_len": 0, "shuffle": false, "repeat": "off",
        "total_tracks": 2, "stop_after_track_id": null, "offset": 0, "limit": 100
    });
    let rendered = render_queue_list(&v);
    assert!(rendered.contains("    1  A"), "{rendered}");
    assert!(rendered.contains("    2  B"), "{rendered}");
    assert!(!rendered.contains("->"), "{rendered}");
}

#[test]
fn render_queue_list_empty_queue_shows_zero_tracks() {
    let v = serde_json::json!({
        "current_track": null, "current_index": null, "upcoming": [],
        "history_len": 0, "shuffle": false, "repeat": "off",
        "total_tracks": 0, "stop_after_track_id": null, "offset": 0, "limit": 100
    });
    let rendered = render_queue_list(&v);
    assert_eq!(rendered, format!("{HEADER}\n0 tracks · shuffle off · repeat off\n"));
}
