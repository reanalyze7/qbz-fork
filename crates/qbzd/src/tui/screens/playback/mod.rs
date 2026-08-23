// crates/qbzd/src/tui/screens/playback/ — the Playback screen (03 §3.3).
//
// Reads three stores at entry (daemon_prefs.streaming_quality, AudioSettings
// quality/fallback rows, PlaybackPreferences) and writes back through the App's
// write_one path. The two spec subtleties, both pure + tested:
//   - the ask→fallback rendering rule (§3.3.2): the select offers only the two
//     concrete values; a stored `ask` renders a note until picked; the TUI never
//     writes `ask`.
//   - `infinite` autoplay (P1 radio) renders read-only until toggled (§3.3.1).
mod editor_input;
mod field_render;
mod fields;
mod input;
mod labels;
mod model;
mod render;
mod save;
#[cfg(test)]
mod tests;

pub use fields::{row_state, visible_fields, PField, StagedPlayback};
pub use model::PlaybackState;
