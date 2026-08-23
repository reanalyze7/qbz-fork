use qbz_models::QueueTrack;

use super::paging::paginate;
use super::row::{display_title, fmt_duration, row_from};
use super::PAGE_SIZE;

fn track(id: u64, title: &str, version: Option<&str>) -> QueueTrack {
    QueueTrack {
        id,
        title: title.to_string(),
        version: version.map(|v| v.to_string()),
        artist: "Artist".to_string(),
        album: "Album".to_string(),
        album_version: None,
        duration_secs: 100,
        artwork_url: None,
        hires: false,
        bit_depth: None,
        sample_rate: None,
        is_local: false,
        album_id: None,
        artist_id: None,
        streamable: true,
        source: None,
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}

#[test]
fn fmt_duration_pads_seconds() {
    assert_eq!(fmt_duration(0), "0:00");
    assert_eq!(fmt_duration(9), "0:09");
    assert_eq!(fmt_duration(65), "1:05");
    assert_eq!(fmt_duration(3725), "62:05");
}

#[test]
fn display_title_appends_version() {
    assert_eq!(display_title(&track(1, "Song", Some("Live"))), "Song (Live)");
    assert_eq!(display_title(&track(1, "Song", None)), "Song");
    // Empty version string is treated as no version.
    assert_eq!(display_title(&track(1, "Song", Some(""))), "Song");
}

#[test]
fn row_from_marks_playing_and_explicit() {
    let mut t = track(7, "Song", Some("Mix"));
    t.parental_warning = true;
    t.duration_secs = 215;
    let row = row_from(&t, true);
    assert_eq!(row.id, "7");
    assert_eq!(row.title, "Song (Mix)");
    assert_eq!(row.duration, "3:35");
    assert!(row.playing);
    assert!(row.explicit);
}

#[test]
fn paginate_single_page() {
    let b = paginate(10, 0);
    assert_eq!(b.page_count, 1);
    assert_eq!(b.page, 0);
    assert_eq!((b.start, b.end), (0, 10));
}

#[test]
fn paginate_exact_page_boundary() {
    // Exactly PAGE_SIZE items -> one full page, not two.
    let b = paginate(PAGE_SIZE, 0);
    assert_eq!(b.page_count, 1);
    assert_eq!((b.start, b.end), (0, PAGE_SIZE));
}

#[test]
fn paginate_spans_multiple_pages() {
    // Two full pages + a short tail -> 3 pages. PAGE_SIZE-relative so the
    // multi-page math is validated regardless of the configured page size
    // (which is now effectively unbounded for the growing-list queue).
    let total = PAGE_SIZE * 2 + 15;
    let p0 = paginate(total, 0);
    assert_eq!(p0.page_count, 3);
    assert_eq!((p0.start, p0.end), (0, PAGE_SIZE));
    let p1 = paginate(total, 1);
    assert_eq!((p1.start, p1.end), (PAGE_SIZE, PAGE_SIZE * 2));
    let p2 = paginate(total, 2);
    assert_eq!((p2.start, p2.end), (PAGE_SIZE * 2, total));
}

#[test]
fn paginate_clamps_overshot_page() {
    // Requesting page 9 of a 2-page list clamps to the last page.
    let total = PAGE_SIZE + 10;
    let b = paginate(total, 9);
    assert_eq!(b.page_count, 2);
    assert_eq!(b.page, 1);
    assert_eq!((b.start, b.end), (PAGE_SIZE, total));
}

#[test]
fn paginate_empty_list_has_one_page() {
    let b = paginate(0, 0);
    assert_eq!(b.page_count, 1);
    assert_eq!((b.start, b.end), (0, 0));
}
