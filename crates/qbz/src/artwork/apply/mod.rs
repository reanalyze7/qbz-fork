//! `apply_artwork` — the top-level function that builds the `slint::Image`
//! once and dispatches by UI area to a thin per-arm helper in each sibling
//! file. Split out because the original single `match` (all ~90
//! `ArtworkTarget` variants) was the single biggest offender in the file.

mod album_misc;
mod artist;
mod cortinilla;
mod extreco;
mod favorites;
mod foryou;
mod home_discover;
mod label;
mod library_local;
mod mix_playlist;
mod myqbz;
mod search_immersive;
mod suggestions_pinned;

use crate::AppWindow;

use super::target::ArtworkTarget;

/// Apply decoded pixels to a single card. Runs on the Slint event loop.
///
/// Each category module's `apply` returns `false` when `target` is not one
/// of the variants it owns, so the dispatcher falls through to the next
/// category; exactly one category ever claims a given target.
pub(in crate::artwork) fn apply_artwork(
    window: &AppWindow,
    target: ArtworkTarget,
    url: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
) {
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);
    let dst = buffer.make_mut_bytes();
    if dst.len() != pixels.len() {
        return;
    }
    dst.copy_from_slice(pixels);
    let image = slint::Image::from_rgba8(buffer);

    if home_discover::apply(window, target.clone(), &image, pixels, width, height) {
        return;
    }
    if search_immersive::apply(window, target.clone(), url, &image) {
        return;
    }
    if cortinilla::apply(window, target.clone(), url, &image) {
        return;
    }
    if artist::apply(window, target.clone(), &image) {
        return;
    }
    if label::apply(window, target.clone(), &image) {
        return;
    }
    if album_misc::apply(window, target.clone(), &image) {
        return;
    }
    if library_local::apply(window, target.clone(), &image) {
        return;
    }
    if favorites::apply(window, target.clone(), &image) {
        return;
    }
    if foryou::apply(window, target.clone(), &image) {
        return;
    }
    if extreco::apply(window, target.clone(), &image) {
        return;
    }
    if mix_playlist::apply(window, target.clone(), &image) {
        return;
    }
    if myqbz::apply(window, target.clone(), &image) {
        return;
    }
    let _ = suggestions_pinned::apply(window, target, &image, pixels, width, height);
}
