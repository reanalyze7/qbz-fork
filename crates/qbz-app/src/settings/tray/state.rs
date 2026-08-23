use super::prefs::TraySettings;
use super::store::TraySettingsStore;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct TraySettingsState {
    pub store: Arc<Mutex<Option<TraySettingsStore>>>,
}

impl TraySettingsState {
    pub fn new() -> Result<Self, String> {
        let store = TraySettingsStore::new()?;
        Ok(Self {
            store: Arc::new(Mutex::new(Some(store))),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_at(&self, base_dir: &Path) -> Result<(), String> {
        let new_store = TraySettingsStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        *guard = None;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<TraySettings, String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.get_settings()
    }

    pub fn set_enable_tray(&self, value: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_enable_tray(value)
    }

    pub fn set_minimize_to_tray(&self, value: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_minimize_to_tray(value)
    }

    pub fn set_close_to_tray(&self, value: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_close_to_tray(value)
    }

    pub fn set_tray_icon_theme(&self, value: &str) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_tray_icon_theme(value)
    }

    pub fn set_mac_hide_dock(&self, value: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock tray settings store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_mac_hide_dock(value)
    }
}
