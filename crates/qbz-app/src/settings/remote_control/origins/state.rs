use std::path::Path;
use std::sync::{Arc, Mutex};

use super::store::AllowedOriginsStore;
use super::AllowedOrigin;

pub struct AllowedOriginsState {
    pub store: Arc<Mutex<Option<AllowedOriginsStore>>>,
}

impl AllowedOriginsState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            store: Arc::new(Mutex::new(Some(AllowedOriginsStore::new()?))),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init_at(&self, base_dir: &Path) -> Result<(), String> {
        let new_store = AllowedOriginsStore::new_at(base_dir)?;
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock allowed origins store".to_string())?;
        *guard = Some(new_store);
        Ok(())
    }

    pub fn teardown(&self) -> Result<(), String> {
        let mut guard = self
            .store
            .lock()
            .map_err(|_| "Failed to lock allowed origins store".to_string())?;
        *guard = None;
        Ok(())
    }

    pub fn get_origins(&self) -> Result<Vec<AllowedOrigin>, String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.get_origins()
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.store
            .lock()
            .map(|guard| {
                guard
                    .as_ref()
                    .map(|store| store.is_origin_allowed(origin))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    pub fn add_origin(&self, origin: &str) -> Result<AllowedOrigin, String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.add_origin(origin)
    }

    pub fn remove_origin(&self, id: i64) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.remove_origin(id)
    }

    pub fn restore_defaults(&self) -> Result<(), String> {
        let guard = self
            .store
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let store = guard.as_ref().ok_or("No active session - please log in")?;
        store.restore_defaults()
    }
}
