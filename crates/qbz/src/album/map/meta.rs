//! The header meta-line assembly: "year • label • genre • N tracks •
//! duration", split around the label so the header can render it as a link.

use qbz_models::Album;

use super::credits::format_release_date;
use super::text::format_duration;

pub(super) struct MetaLine {
    /// Segment BEFORE the label (the year) — rendered with the label as a
    /// clickable link in the header.
    pub(super) pre: String,
    /// Segment AFTER the label (genre • N tracks • duration).
    pub(super) post: String,
    /// The full line (label inlined as plain text) — the fallback for when
    /// there is no label id to navigate to.
    pub(super) full: String,
}

pub(super) fn build_meta_line(album: &Album) -> MetaLine {
    // Full readable release date ("Feb 19, 2026"); was year-only before.
    // Prefer the flat ISO field, fall back to the nested V2 `dates.original`.
    let date_display = format_release_date(
        album
            .release_date_original
            .as_deref()
            .or_else(|| album.dates.as_ref().and_then(|d| d.original.as_deref())),
    );
    let label_name = album
        .label
        .as_ref()
        .filter(|l| !l.name.is_empty())
        .map(|l| l.name.clone());
    let genre_str = album
        .genre
        .as_ref()
        .filter(|g| !g.name.is_empty())
        .map(|g| g.name.clone());
    let tracks_str = album.tracks_count.map(|count| {
        qbz_i18n::tf("{} track", "{} tracks", count as i64, &[&count.to_string()])
    });
    let duration_str = album.duration.map(format_duration);

    let mut pre_parts: Vec<String> = Vec::new();
    if !date_display.is_empty() {
        pre_parts.push(date_display.clone());
    }
    let mut post_parts: Vec<String> = Vec::new();
    if let Some(g) = &genre_str {
        post_parts.push(g.clone());
    }
    if let Some(tc) = &tracks_str {
        post_parts.push(tc.clone());
    }
    if let Some(d) = &duration_str {
        post_parts.push(d.clone());
    }

    let pre = pre_parts.join("   •   ");
    let post = post_parts.join("   •   ");

    let mut all_parts = pre_parts.clone();
    if let Some(l) = &label_name {
        all_parts.push(l.clone());
    }
    all_parts.extend(post_parts.clone());
    let full = all_parts.join("   •   ");

    MetaLine { pre, post, full }
}
