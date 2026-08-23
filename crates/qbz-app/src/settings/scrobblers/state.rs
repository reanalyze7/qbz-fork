use super::store::ScrobblerSettingsStore;
use super::ScrobblerSettings;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct ScrobblerSettingsState {
    pub store: Arc<Mutex<Option<ScrobblerSettingsStore>>>,
}

impl Default for ScrobblerSettingsState {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl ScrobblerSettingsState {
    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_at(&self, base_dir: &Path) -> Result<(), String> {
        let new_store = ScrobblerSettingsStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock scrobbler settings store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock scrobbler settings store".to_string())?;
        *guard = None;
        Ok(())
    }

    fn with_store<T>(
        &self,
        f: impl FnOnce(&ScrobblerSettingsStore) -> Result<T, String>,
    ) -> Result<T, String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock scrobbler settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        f(store)
    }

    pub fn get_settings(&self) -> Result<ScrobblerSettings, String> {
        self.with_store(|s| s.get_settings())
    }

    pub fn set_enabled(&self, value: bool) -> Result<(), String> {
        self.with_store(|s| s.set_enabled(value))
    }

    pub fn set_ui_collapsed(&self, value: bool) -> Result<(), String> {
        self.with_store(|s| s.set_ui_collapsed(value))
    }

    pub fn set_lastfm_enabled(&self, value: bool) -> Result<(), String> {
        self.with_store(|s| s.set_lastfm_enabled(value))
    }

    pub fn set_lastfm_session(&self, key: &str, username: &str) -> Result<(), String> {
        self.with_store(|s| s.set_lastfm_session(key, username))
    }

    pub fn disconnect_lastfm(&self) -> Result<(), String> {
        self.with_store(|s| s.disconnect_lastfm())
    }

    pub fn set_listenbrainz_enabled(&self, value: bool) -> Result<(), String> {
        self.with_store(|s| s.set_listenbrainz_enabled(value))
    }

    pub fn set_listenbrainz_token(&self, token: &str, username: &str) -> Result<(), String> {
        self.with_store(|s| s.set_listenbrainz_token(token, username))
    }

    pub fn disconnect_listenbrainz(&self) -> Result<(), String> {
        self.with_store(|s| s.disconnect_listenbrainz())
    }
}
