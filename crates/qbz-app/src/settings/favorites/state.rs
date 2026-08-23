use super::store::FavoritesPreferencesStore;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct FavoritesPreferencesState {
    pub store: Arc<Mutex<Option<FavoritesPreferencesStore>>>,
}

impl FavoritesPreferencesState {
    pub fn new() -> Result<Self, String> {
        let store = FavoritesPreferencesStore::new()?;
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
        let new_store = FavoritesPreferencesStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock favorites preferences store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock favorites preferences store".to_string())?;
        *guard = None;
        Ok(())
    }
}
