//! Hero-mosaic cover-count derivation + decoded-cover setters.

use qbz_models::mixtape::{AlbumSource, CollectionKind, MixtapeCollection};

use crate::MyQbzDetailState;

/// Decide the hero-mosaic cover-count (0 / 4 / 9) + downscaled cell URLs, and
/// push them into `MyQbzDetailState`. Mirrors the grid card's mosaic rule
/// (3x3 only for a Collection with >= 9 items; else 2x2) but at the hero
/// `size = 186` (so the downscale target differs: 2x2 -> 150, 3x3 -> 50).
pub(super) fn apply_hero_mosaic(state: &MyQbzDetailState, c: &MixtapeCollection) {
    let item_count = c.items.len();
    let has_custom = c.custom_artwork_path.is_some();

    let cols: usize = if c.kind == CollectionKind::Collection && item_count >= 9 {
        3
    } else {
        2
    };
    let cell_count = cols * cols;
    let cover_count = if has_custom || item_count == 0 {
        0
    } else {
        cell_count
    };
    // Hero renders at 186px; cell ~93 (2x2) -> 150, ~62 (3x3) -> 50.
    let target: u32 = if cols == 3 { 50 } else { 150 };

    let url = |i: usize| -> slint::SharedString {
        if has_custom || item_count == 0 || i >= cell_count {
            return slint::SharedString::default();
        }
        let Some(it) = c.items.get(i) else {
            return slint::SharedString::default();
        };
        match it.artwork_url.as_deref() {
            // `small_qobuz_url` is Qobuz-CDN-specific; only rewrite Qobuz cells.
            // Local artwork paths pass through raw for the source-aware
            // dispatch.
            Some(u) if !u.is_empty() && it.source == AlbumSource::Qobuz => {
                crate::myqbz::small_qobuz_url(u, target).into()
            }
            Some(u) if !u.is_empty() => u.to_string().into(),
            _ => slint::SharedString::default(),
        }
    };

    state.set_cover_count(cover_count as i32);
    state.set_url1(url(0));
    state.set_url2(url(1));
    state.set_url3(url(2));
    state.set_url4(url(3));
    state.set_url5(url(4));
    state.set_url6(url(5));
    state.set_url7(url(6));
    state.set_url8(url(7));
    state.set_url9(url(8));
    // Reset the decoded covers so a re-open does not show stale tiles.
    state.set_cover1(slint::Image::default());
    state.set_cover2(slint::Image::default());
    state.set_cover3(slint::Image::default());
    state.set_cover4(slint::Image::default());
    state.set_cover5(slint::Image::default());
    state.set_cover6(slint::Image::default());
    state.set_cover7(slint::Image::default());
    state.set_cover8(slint::Image::default());
    state.set_cover9(slint::Image::default());
}

/// Set a decoded hero-mosaic cover by slot (0-8).
pub fn set_hero_cover(window: &crate::AppWindow, slot: usize, image: slint::Image) {
    use slint::ComponentHandle;
    let state = window.global::<MyQbzDetailState>();
    match slot {
        0 => state.set_cover1(image),
        1 => state.set_cover2(image),
        2 => state.set_cover3(image),
        3 => state.set_cover4(image),
        4 => state.set_cover5(image),
        5 => state.set_cover6(image),
        6 => state.set_cover7(image),
        7 => state.set_cover8(image),
        8 => state.set_cover9(image),
        _ => {}
    }
}
