//! Settings controller — Audio and Playback preferences.
//!
//! Owns the two persistence stores (`AudioSettingsStore` from `qbz-audio`,
//! `PlaybackPreferencesStore` from `qbz-app`) plus the JSON `ui_prefs`
//! store (Streaming Quality), and bridges them to the `SettingsState`
//! Slint global.
//!
//! Audio changes are persisted and then applied to the live `Player`:
//! routing-critical changes (backend, output device, exclusive mode,
//! DAC passthrough, ALSA plugin) trigger a device re-init; the rest only
//! reload the settings struct. Playback-preference changes (autoplay,
//! show-context, persist, resume) just persist.
//!
//! Neither domain store is exposed by `AppRuntime`, so this module opens
//! them directly at the shared global path — the same path
//! `AppRuntime::new` reads to seed the `Player`, so the two stay
//! consistent.

mod apply;
mod devices;
mod export;
mod handlers;
mod snapshot;
mod store;
mod tables;

pub use apply::{apply_startup_bitperfect_volume, refresh_device_cap};
pub use export::export_settings;
pub use handlers::{handle_bool, handle_release_device, handle_reset, handle_select, handle_slider, handle_string};
pub use snapshot::{apply_snapshot, load_snapshot, SettingsSnapshot};
pub use store::SettingsCtx;
