//! Runtime-registered live-secret storage (distinct from the compiled static patterns).

use std::sync::{OnceLock, RwLock};

pub(super) fn secrets() -> &'static RwLock<Vec<String>> {
    static SECRETS: OnceLock<RwLock<Vec<String>>> = OnceLock::new();
    SECRETS.get_or_init(|| RwLock::new(Vec::new()))
}
