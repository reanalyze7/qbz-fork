//! Thread-safe wrapper around an optional `AudioSettingsStore`.

use super::store_core::AudioSettingsStore;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct AudioSettingsState {
    pub store: Arc<Mutex<Option<AudioSettingsStore>>>,
}

impl AudioSettingsState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            store: Arc::new(Mutex::new(Some(AudioSettingsStore::new()?))),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_at(&self, base_dir: &Path) -> Result<(), String> {
        let new_store = AudioSettingsStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock audio settings store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock audio settings store".to_string())?;
        *guard = None;
        Ok(())
    }
}

impl Default for AudioSettingsState {
    fn default() -> Self {
        Self::new_empty()
    }
}
