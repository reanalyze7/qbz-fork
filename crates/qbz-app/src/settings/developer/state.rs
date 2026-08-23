use super::DeveloperSettingsStore;
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper for host state management.
pub struct DeveloperSettingsState {
    pub store: Arc<Mutex<Option<DeveloperSettingsStore>>>,
}

impl DeveloperSettingsState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            store: Arc::new(Mutex::new(Some(DeveloperSettingsStore::new()?))),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for DeveloperSettingsState {
    fn default() -> Self {
        Self::new_empty()
    }
}
