//! Now-playing cover resolution: the bar-size decode and the higher-res
//! decode that also feeds the immersive-view ambient background/glow.
use slint::ComponentHandle;

use crate::{AppWindow, ImmersiveState, NowPlayingState};

/// Resolve the now-playing cover and apply it to `NowPlayingState`.
///
/// Takes a source-aware [`qbz_models::ArtworkRef`] so local-library covers
/// reach the now-playing bar, not just remote Qobuz URLs.
pub(super) fn load_now_playing_artwork(weak: slint::Weak<AppWindow>, art: qbz_models::ArtworkRef) {
    if art.is_empty() {
        return;
    }
    let Some(cache) = crate::artwork::shared_cache() else {
        return;
    };
    tokio::spawn(async move {
        let Some((pixels, w, h)) = crate::artwork::fetch_and_decode_ref(&art, &cache, 160).await
        else {
            return;
        };
        let _ = weak.upgrade_in_event_loop(move |win| {
            let img = crate::artwork::pixels_to_image(&pixels, w, h);
            win.global::<NowPlayingState>().set_artwork(img);
        });
    });
}

/// Resolve a HIGHER-res now-playing cover (~300px) and apply it to
/// `NowPlayingState.artwork-large`. Feeds the hover preview that floats above
/// the bar's small art, so the ~220px popup is crisp instead of an upscale of
/// the 160px bar art. Mirrors [`load_now_playing_artwork`] but decodes larger
/// and writes the separate `artwork-large` slot. Source-aware via the SAME
/// `ArtworkRef` funnel (the caller passes a `Some(300)` ref).
pub(super) fn load_now_playing_artwork_large(
    weak: slint::Weak<AppWindow>,
    art: qbz_models::ArtworkRef,
) {
    if art.is_empty() {
        return;
    }
    let Some(cache) = crate::artwork::shared_cache() else {
        return;
    };
    tokio::spawn(async move {
        // Decode at high resolution (was 300 — the hover-preview size). The
        // IMMERSIVE view grows this cover to 600-800px, so a 300px decode was
        // upscaled and looked blurry on large windows. Decoding up to 1000px lets
        // the FULL source resolution through (Qobuz typically serves ~600); the
        // immersive's native-size cap then grows the art sharply up to that real
        // resolution instead of upscaling a tiny decode.
        let Some((pixels, w, h)) = crate::artwork::fetch_and_decode_ref(&art, &cache, 1000).await
        else {
            return;
        };
        // ALL pixel crunching stays HERE, off the UI thread. The decode is up
        // to 1000px, and the four cover-derived visuals below each used to run
        // their own full-size-to-tiny resize (plus a full buffer copy) INSIDE
        // the event loop — a visible stall at every track boundary on weak
        // hardware. Only the finished Send carriers (SharedPixelBuffer +
        // Colors) cross into the event loop; the !Send slint::Image is built
        // there, matching the pixels_to_image pattern.

        // Full-res cover buffer for the hover preview / immersive cover.
        // Mirrors pixels_to_image's length guard (None → empty image).
        let artwork_buf = {
            let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
            let dst = buf.make_mut_bytes();
            if dst.len() == pixels.len() {
                dst.copy_from_slice(&pixels);
                Some(buf)
            } else {
                None
            }
        };

        // One shared downscale (8x8 + 16x16, consuming `pixels` — no copy)
        // feeds atmosphere + glow + spectrum + lyrics accent with the exact
        // sampling inputs each helper used to compute for itself.
        let analysis = crate::immersive::cover_tiny_samples(pixels, w, h).map(|(tiny8, tiny16)| {
            let (bg_pixels, bg_w, bg_h) = crate::immersive::atmosphere_from_tiny8(&tiny8);
            let bg_buf = {
                let mut buf = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(bg_w, bg_h);
                let dst = buf.make_mut_bytes();
                if dst.len() == bg_pixels.len() {
                    dst.copy_from_slice(&bg_pixels);
                    Some(buf)
                } else {
                    None
                }
            };
            let glow = crate::immersive::glow_color(&tiny8);
            let (spec_primary, spec_secondary) = crate::immersive::spectrum_colors(&tiny16);
            let spec_accent = crate::immersive::lyrics_accent_color(&tiny16);
            (bg_buf, glow, spec_primary, spec_secondary, spec_accent)
        });

        let _ = weak.upgrade_in_event_loop(move |win| {
            let img = match artwork_buf {
                Some(buf) => slint::Image::from_rgba8(buf),
                None => slint::Image::default(),
            };
            // The hover-preview cover is ALWAYS needed (independent of the
            // immersive overlay), so set it unconditionally first.
            win.global::<NowPlayingState>().set_artwork_large(img);

            // ALWAYS (re)apply the immersive ambient atmosphere (Codex's
            // blurred moving background) + glow + spectrum colors from this cover.
            // The overlay is conditionally mounted so applying while closed is
            // cheap; the track-change reset clears bg-image, so this MUST run
            // unconditionally — a URL dedupe here left bg-image empty after the
            // reset and the atmosphere fell back to the raw (sharp) cover.
            if let Some((bg_buf, glow, spec_primary, spec_secondary, spec_accent)) = analysis {
                let imm = win.global::<ImmersiveState>();
                if let Some(bg_buf) = bg_buf {
                    imm.set_bg_image(slint::Image::from_rgba8(bg_buf));
                }
                imm.set_glow_color(glow);
                imm.set_spectrum_primary(spec_primary);
                imm.set_spectrum_secondary(spec_secondary);
                imm.set_lyrics_accent(spec_accent);
                // Feed the same album-art triad to the wgpu shader underlay so the
                // immersive shaders (Plasma/Tunnel/Aurora) are album-colored instead
                // of hardcoded. Pushed on track change; read on every shader frame
                // (thread-local — must be written on the UI thread, so it stays here).
                crate::shader_underlay::set_palette(spec_primary, spec_secondary, spec_accent);
            }
        });
    });
}
