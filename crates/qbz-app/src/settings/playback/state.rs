use super::store::PlaybackPreferencesStore;
use super::types::{AutoplayMode, PlaybackPreferences};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct PlaybackPreferencesState {
    pub store: Arc<Mutex<Option<PlaybackPreferencesStore>>>,
}

impl PlaybackPreferencesState {
    pub fn new() -> Result<Self, String> {
        let store = PlaybackPreferencesStore::new()?;
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
        let new_store = PlaybackPreferencesStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        *guard = None;
        Ok(())
    }

    pub fn get_preferences(&self) -> Result<PlaybackPreferences, String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.get_preferences()
    }

    pub fn set_autoplay_mode(&self, mode: AutoplayMode) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_autoplay_mode(mode)
    }

    pub fn set_show_context_icon(&self, show: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_show_context_icon(show)
    }

    pub fn set_persist_session(&self, persist: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_persist_session(persist)
    }

    pub fn set_resume_playback_position(&self, resume: bool) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock playback preferences store".to_string())?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.set_resume_playback_position(resume)
    }
}
