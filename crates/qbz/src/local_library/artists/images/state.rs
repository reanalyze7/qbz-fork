//! Generation guard + portrait paint for the Artists rail.

use std::sync::atomic::{AtomicU64, Ordering};

use slint::{ComponentHandle, Model};

use crate::{AppWindow, LocalLibraryState};

/// Generation guard for the artist-image fetch: bumped on every artists load
/// so a stale in-flight fetch/decode (from a superseded list) is dropped.
pub(crate) static ARTISTS_IMG_GEN: AtomicU64 = AtomicU64::new(0);

/// True if `gen` is still the current artist-image generation (the apply arm
/// checks this before painting a portrait).
pub fn artists_img_gen_current() -> u64 {
    ARTISTS_IMG_GEN.load(Ordering::SeqCst)
}

/// Set a freshly-decoded portrait (by artist `name`) on BOTH the flat master
/// (`artists`, so a later `derive_artists` carries it forward) and every
/// rendered grouped section (`artists-grouped[*].artists`). Mirrors
/// `favorites::set_album_artwork`. The `String` name lives here, not in the
/// `Copy` artwork target.
pub fn set_artist_row_image(window: &AppWindow, name: &str, image: slint::Image) {
    let s = window.global::<LocalLibraryState>();
    let flat = s.get_artists();
    for i in 0..flat.row_count() {
        if let Some(mut it) = flat.row_data(i) {
            if it.name.as_str() == name {
                it.image = image.clone();
                flat.set_row_data(i, it);
                break;
            }
        }
    }
    let grouped = s.get_artists_grouped();
    for sx in 0..grouped.row_count() {
        if let Some(sec) = grouped.row_data(sx) {
            for r in 0..sec.artists.row_count() {
                if let Some(mut it) = sec.artists.row_data(r) {
                    if it.name.as_str() == name {
                        it.image = image.clone();
                        sec.artists.set_row_data(r, it);
                        break;
                    }
                }
            }
        }
    }
}
