//! Model builders — the one file that touches `slint::Image`/decoding
//! directly.

use qbz_models::mixtape::{CollectionKind, MixtapeCollection};

use crate::MixtapeCardItem;

use super::labels::{album_count_label, cell_target, kind_str, label_for, small_qobuz_url};

/// Build one ready-to-render card. Decides cover-count (0 / 4 / 9) per the
/// 2x2-vs-3x3 rule, and pre-downscales the up-to-9 cover URLs per cell.
pub(super) fn card_item(c: &MixtapeCollection) -> MixtapeCardItem {
    let item_count = c.items.len();

    // Decode the custom cover from disk so a custom-art mixtape/collection
    // renders its real image in the grid (NOT a blank square). Same source-aware
    // load as the detail view (the path is the on-disk artwork-cache file). A
    // missing/undecodable path is treated as "no custom cover" so the mosaic
    // shows instead of an empty full-bleed square — and so `has_custom` drives
    // cover_count + the URL closure consistently below.
    // Decoded to the card tier (the grid card renders at 184px) so a rebuild
    // per keystroke/sort never retains full-resolution sources; the decoded-
    // pixel cache makes the repeats a lookup.
    let decoded_custom = c
        .custom_artwork_path
        .as_ref()
        .filter(|p| !p.is_empty())
        .filter(|p| std::path::Path::new(p).exists())
        .and_then(|p| crate::artwork::load_local_cover(p, 264));
    let (custom_image, has_custom) = match decoded_custom {
        Some(img) => (img, true),
        None => (slint::Image::default(), false),
    };

    // cols: 3x3 only for a Collection with >= 9 items; else 2x2.
    let cols: u32 = if c.kind == CollectionKind::Collection && item_count >= 9 {
        3
    } else {
        2
    };
    let cell_count = (cols * cols) as usize;
    // cover-count is the number of mosaic cells actually used (0 when empty or
    // when a custom cover full-bleeds; the view checks has-custom-cover first).
    let cover_count = if has_custom || item_count == 0 {
        0
    } else {
        cell_count
    };

    // Grid-card mosaic renders at 184px; size the downscale to that.
    let target = cell_target(184, cols);

    // Up-to-9 cell URLs: the first `cell_count` items' artwork, padded "".
    let url = |i: usize| -> slint::SharedString {
        if has_custom || item_count == 0 || i >= cell_count {
            return slint::SharedString::default();
        }
        match c.items.get(i).and_then(|it| it.artwork_url.as_deref()) {
            Some(u) if !u.is_empty() => small_qobuz_url(u, target).into(),
            _ => slint::SharedString::default(),
        }
    };

    MixtapeCardItem {
        id: c.id.clone().into(),
        name: c.name.clone().into(),
        kind: kind_str(c.kind).into(),
        label: label_for(c.kind).into(),
        meta: album_count_label(item_count).into(),
        item_count: item_count as i32,
        play_count: c.play_count,
        updated_at: c.updated_at as i32,
        custom_cover: custom_image,
        has_custom_cover: has_custom,
        cover_count: cover_count as i32,
        url1: url(0),
        url2: url(1),
        url3: url(2),
        url4: url(3),
        url5: url(4),
        url6: url(5),
        url7: url(6),
        url8: url(7),
        url9: url(8),
        cover1: slint::Image::default(),
        cover2: slint::Image::default(),
        cover3: slint::Image::default(),
        cover4: slint::Image::default(),
        cover5: slint::Image::default(),
        cover6: slint::Image::default(),
        cover7: slint::Image::default(),
        cover8: slint::Image::default(),
        cover9: slint::Image::default(),
    }
}

/// Set a decoded mosaic cover onto a card item by slot (0-8). Called from the
/// artwork apply arm.
pub fn set_mosaic_cover(item: &mut MixtapeCardItem, slot: usize, image: slint::Image) {
    match slot {
        0 => item.cover1 = image,
        1 => item.cover2 = image,
        2 => item.cover3 = image,
        3 => item.cover4 = image,
        4 => item.cover5 = image,
        5 => item.cover6 = image,
        6 => item.cover7 = image,
        7 => item.cover8 = image,
        8 => item.cover9 = image,
        _ => {}
    }
}
