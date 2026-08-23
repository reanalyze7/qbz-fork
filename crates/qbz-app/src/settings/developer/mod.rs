//! Developer settings persistence.
//!
//! This module stores portable developer-mode toggles only. Tauri command
//! wrappers and restart messaging stay outside `qbz-app`.

mod state;
mod store;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use state::DeveloperSettingsState;
pub use store::DeveloperSettingsStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperSettings {
    pub force_dmabuf: bool,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            force_dmabuf: false,
        }
    }
}
