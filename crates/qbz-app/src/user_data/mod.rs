//! Per-user data path management.
//!
//! Each Qobuz user gets their own subdirectory under the app's data/cache paths.
//! This module provides the central path provider that host shells use to
//! determine where to store per-user databases and cache files.

mod last_user;
mod scoped_paths;
#[cfg(test)]
mod tests;

use std::sync::RwLock;

/// Central path provider for per-user data isolation.
///
/// Holds the current user_id and provides methods to get user-scoped data and
/// cache directories.
pub struct UserDataPaths {
    user_id: RwLock<Option<u64>>,
}

impl Default for UserDataPaths {
    fn default() -> Self {
        Self::new()
    }
}
