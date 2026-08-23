use std::path::Path;
use std::sync::{Arc, Mutex};

use super::store::RemoteControlSettingsStore;
use super::RemoteControlSettings;

pub struct RemoteControlSettingsState {
    pub store: Arc<Mutex<Option<RemoteControlSettingsStore>>>,
}

impl RemoteControlSettingsState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            store: Arc::new(Mutex::new(Some(RemoteControlSettingsStore::new()?))),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_at(&self, base_dir: &Path) -> Result<(), String> {
        let new_store = RemoteControlSettingsStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock remote control settings store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock remote control settings store".to_string())?;
        *guard = None;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<RemoteControlSettings, String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.get_settings()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_enabled(enabled)
    }

    pub fn set_port(&self, port: u16) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_port(port)
    }

    pub fn set_secure(&self, secure: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_secure(secure)
    }

    pub fn regenerate_token(&self) -> Result<String, String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.regenerate_token()
    }
}
