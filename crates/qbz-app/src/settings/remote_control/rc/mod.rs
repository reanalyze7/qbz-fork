mod setters;
mod state;
mod store;

use serde::{Deserialize, Serialize};

pub use state::RemoteControlSettingsState;
pub use store::RemoteControlSettingsStore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteControlSettings {
    pub enabled: bool,
    pub port: u16,
    pub secure: bool,
    pub token: String,
}

impl Default for RemoteControlSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8182,
            secure: true,
            token: String::new(),
        }
    }
}
