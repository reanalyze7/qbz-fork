//! The persisted `Prefs` shape + its defaults.

use serde::{Deserialize, Serialize};

/// The five persisted view-pref fields for one collection (spec 12 §18). Source
/// filter is the three independent flags the Slint toolbar uses (Slint has no
/// Set); together they round-trip the Tauri `sourceFilter:[SourceKind]` array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default = "d_list")]
    pub view_mode: String,
    #[serde(default = "d_position")]
    pub sort_by: String,
    #[serde(default = "d_asc")]
    pub sort_dir: String,
    #[serde(default = "d_all")]
    pub type_filter: String,
    #[serde(default)]
    pub src_qobuz: bool,
    #[serde(default)]
    pub src_local: bool,
}

fn d_list() -> String {
    "list".to_string()
}
fn d_position() -> String {
    "position".to_string()
}
fn d_asc() -> String {
    "asc".to_string()
}
fn d_all() -> String {
    "all".to_string()
}

impl Default for Prefs {
    /// The §18 defaults: list / position / asc / all / empty source set.
    fn default() -> Self {
        Self {
            view_mode: d_list(),
            sort_by: d_position(),
            sort_dir: d_asc(),
            type_filter: d_all(),
            src_qobuz: false,
            src_local: false,
        }
    }
}
