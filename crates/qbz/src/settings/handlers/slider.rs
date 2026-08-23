//! `handle_slider` — Initial Buffer Size and Crossfade sliders.

use qbz_app::shell::AppRuntime;

use crate::adapter::SlintAdapter;
use crate::settings::apply::apply_audio;
use crate::settings::store::{with_audio, Apply, SettingsCtx};

/// Handle a slider change: persist it and reload the player settings.
/// Currently only the Initial Buffer Size slider exists.
pub fn handle_slider(ctx: &SettingsCtx, runtime: &AppRuntime<SlintAdapter>, key: &str, value: i32) {
    match key {
        "buffer-seconds" => {
            let seconds = value.clamp(1, 10) as u8;
            match with_audio(&ctx.audio, |s| s.set_stream_buffer_seconds(seconds)) {
                Ok(()) => apply_audio(ctx, runtime, Apply::Reload),
                Err(e) => log::error!("[qbz-slint] persist buffer seconds failed: {e}"),
            }
        }
        "crossfade-seconds" => {
            let seconds = value.clamp(0, 10) as f32;
            match with_audio(&ctx.audio, |s| s.set_crossfade_seconds(seconds)) {
                // Apply::Reload re-reads AudioSettings into the live player,
                // which is where the gapless PlayNext handler reads
                // `crossfade_seconds` from on every track transition — no
                // separate live-apply path needed, same as buffer-seconds.
                Ok(()) => apply_audio(ctx, runtime, Apply::Reload),
                Err(e) => log::error!("[qbz-slint] persist crossfade seconds failed: {e}"),
            }
        }
        other => log::warn!("[qbz-slint] unknown settings slider key: {other}"),
    }
}
