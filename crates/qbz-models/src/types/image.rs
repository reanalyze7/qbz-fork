//! Image set with multiple resolutions.

use serde::{Deserialize, Serialize};

/// Image set with multiple resolutions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageSet {
    pub small: Option<String>,
    pub thumbnail: Option<String>,
    pub large: Option<String>,
    pub extralarge: Option<String>,
    pub mega: Option<String>,
    pub back: Option<String>,
}

impl ImageSet {
    pub fn best(&self) -> Option<&String> {
        self.mega
            .as_ref()
            .or(self.extralarge.as_ref())
            .or(self.large.as_ref())
            .or(self.thumbnail.as_ref())
            .or(self.small.as_ref())
    }

    /// The smallest available variant — for list-row thumbnails, where
    /// `best()` (mega/large) would needlessly download huge images.
    pub fn smallest(&self) -> Option<&String> {
        self.small
            .as_ref()
            .or(self.thumbnail.as_ref())
            .or(self.large.as_ref())
            .or(self.extralarge.as_ref())
            .or(self.mega.as_ref())
    }
}
