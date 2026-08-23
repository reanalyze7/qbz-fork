use crate::tui::widgets::{control_column, follow_scroll, sidebar_is_wide, sidebar_width, wrap};

// ---- sidebar_width: 28 wide, 14 compact, with the 100-col boundary ----

#[test]
fn sidebar_width_doubles_only_at_and_above_100_cols() {
    assert_eq!(sidebar_width(80), 14, "the 80x24 floor keeps the compact sidebar");
    assert_eq!(sidebar_width(99), 14, "just below the boundary stays compact");
    assert_eq!(sidebar_width(100), 28, "at 100 the sidebar at least doubles");
    assert_eq!(sidebar_width(120), 28);
    assert!(sidebar_is_wide(120) && !sidebar_is_wide(80));
    // The wide sidebar is at least double the compact one (owner's ask).
    assert!(sidebar_width(120) >= 2 * sidebar_width(80));
}

// ---- wrap(): word boundaries, hard-split, edges ----

#[test]
fn wrap_empty_and_whitespace_yield_no_lines() {
    assert!(wrap("", 10).is_empty());
    assert!(wrap("   ", 10).is_empty());
    assert!(wrap("\n\n", 10).is_empty());
}

#[test]
fn wrap_keeps_short_text_on_one_line() {
    assert_eq!(wrap("short note", 20), vec!["short note".to_string()]);
    // Exact-fit boundary: width == text length stays one line.
    assert_eq!(wrap("exactly ten", 11), vec!["exactly ten".to_string()]);
}

#[test]
fn wrap_breaks_on_word_boundaries_not_mid_word() {
    // "anyone on your network" at width 12 wraps between words.
    let out = wrap("anyone on your network can control playback", 12);
    assert!(out.iter().all(|l| l.chars().count() <= 12), "no line exceeds width: {out:?}");
    // No word is split (every input word survives intact somewhere).
    for word in "anyone on your network can control playback".split(' ') {
        assert!(out.iter().any(|l| l.split(' ').any(|w| w == word)), "word {word:?} intact");
    }
}

#[test]
fn wrap_hard_splits_a_word_longer_than_width() {
    // A 20-char token at width 8 is the only case a word breaks.
    let out = wrap("supercalifragilistic", 8);
    assert!(out.len() >= 3);
    assert!(out.iter().all(|l| l.chars().count() <= 8));
    assert_eq!(out.concat(), "supercalifragilistic", "no characters are lost");
}

#[test]
fn wrap_honors_embedded_newlines_as_hard_breaks() {
    let out = wrap("line one\nline two", 40);
    assert_eq!(out, vec!["line one".to_string(), "line two".to_string()]);
}

// ---- control_column: one mechanism, consistent + clamped ----

#[test]
fn control_column_is_longest_label_plus_gutter_clamped() {
    // 2 indent + 17-char label + 2 gap = 21.
    assert_eq!(control_column(&["Streaming quality", "Gapless"], 62), 21);
    // A short label set still lands at the floor of 14.
    assert_eq!(control_column(&["Port"], 62), 14);
    // A pathological label is clamped so the control keeps room.
    let ceiling = 30u16.saturating_sub(12).max(14);
    assert_eq!(control_column(&["a very very very long label here"], 30), ceiling);
}

// ---- follow_scroll: keep the focused block visible, minimally ----

#[test]
fn follow_scroll_no_scroll_when_everything_fits() {
    assert_eq!(follow_scroll(0, 3, 18, 12), 0);
    assert_eq!(follow_scroll(9, 3, 18, 12), 0);
}

#[test]
fn follow_scroll_brings_a_block_below_the_fold_into_view() {
    // total 30, viewport 18. A block at [20,23) needs scroll so its bottom (23)
    // sits at the viewport bottom: 23 - 18 = 5.
    assert_eq!(follow_scroll(20, 3, 18, 30), 5);
    // Clamped to max_scroll = 30 - 18 = 12.
    assert_eq!(follow_scroll(29, 3, 18, 30), 12);
}

#[test]
fn follow_scroll_pins_top_of_a_block_taller_than_the_viewport() {
    // A 25-row block starting at 4, viewport 18 → pin to the block top (4).
    assert_eq!(follow_scroll(4, 25, 18, 40), 4);
}
