// crates/qbzd/src/tui/screens/audio/ — the Audio screen (03-setup-tui.md §3.2).
//
// The J1-critical screen. Writes audio_settings.db in the daemon data root via
// AudioSettingsStore::new_at (reused through the App's write_one path — no new
// persistence). The three load-bearing PURE pieces (unit-tested):
//   1. the constraint matrix (§3.2.3 shown/enabled) — `row_state`;
//   2. the cross-setting cascades (§3.2.3 items 1-7) — `cascade_*`;
//   3. the device picker grouping (§3.2.2), re-derived from the desktop
//      `crates/qbz/src/settings.rs` rules (we must NOT depend on the qbz bin
//      crate — it pulls qbz-ui). `group_devices` reproduces `alsa_section` /
//      `device_is_bit_perfect` / `group_alsa_devices` 1:1, including the
//      is_default-vs-section badge edge case.
mod cascades;
mod device_grouping;
mod editor_input;
mod field_render;
mod fields;
mod input;
mod labels;
mod model;
mod pickers;
mod render;
mod save;
mod state;
#[cfg(test)]
mod tests;

pub use device_grouping::{group_devices, DeviceEntry};
pub use state::AudioState;
