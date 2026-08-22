//! Offline cache path validation and management
//!
//! Handles path validation, permission checking, and mount status
//! verification, plus migrating cached files when the user relocates the
//! cache. Split into `validate` (one-shot checks), `migrate` (relocation),
//! and `mount_cache` (the 30s-TTL memoized mount check).

mod migrate;
mod mount_cache;
mod validate;

pub use migrate::{move_cached_files_to_new_path, MoveReport};
pub use mount_cache::is_offline_root_available;
pub use validate::{check_mount_status, check_permissions, validate_path, PathStatus, PathValidationResult};
