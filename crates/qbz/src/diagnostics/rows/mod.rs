//! Row builders (1:1 with DiagnosticsPanel.svelte) — the Slint `DiagRow`
//! lists for each panel section, plus the small formatting helpers `report`
//! reuses for the markdown export.

mod audio;
mod env;
mod graphics;
mod helpers;
mod playback;
mod system;

pub(super) use audio::build_audio_rows;
pub(super) use env::build_env_rows;
pub(super) use graphics::build_graphics_rows;
pub(super) use helpers::{match_status, opt, trim_khz, yn};
pub(super) use playback::build_playback_rows;
pub(super) use system::build_system_rows;
