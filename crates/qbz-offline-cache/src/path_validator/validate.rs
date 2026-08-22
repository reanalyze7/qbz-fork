//! Validate a candidate offline cache root path: exists, is a directory,
//! is mounted, and is writable.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathStatus {
    Valid,
    DoesNotExist,
    NotADirectory,
    NoWritePermission,
    NotMounted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValidationResult {
    pub status: PathStatus,
    pub message: String,
}

/// Validate an offline cache path
pub fn validate_path(path: &str) -> Result<PathValidationResult, String> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Ok(PathValidationResult {
            status: PathStatus::DoesNotExist,
            message: format!("Path does not exist: {}", path),
        });
    }

    if !path_obj.is_dir() {
        return Ok(PathValidationResult {
            status: PathStatus::NotADirectory,
            message: format!("Path is not a directory: {}", path),
        });
    }

    if !check_mount_status(path)? {
        return Ok(PathValidationResult {
            status: PathStatus::NotMounted,
            message: "Storage device is not mounted".to_string(),
        });
    }

    if !check_permissions(path)? {
        return Ok(PathValidationResult {
            status: PathStatus::NoWritePermission,
            message: "No write permission for this directory".to_string(),
        });
    }

    Ok(PathValidationResult {
        status: PathStatus::Valid,
        message: "Path is valid and writable".to_string(),
    })
}

/// Check if we have write permissions on a path
pub fn check_permissions(path: &str) -> Result<bool, String> {
    let path_obj = Path::new(path);
    let test_file = path_obj.join(".qbz_write_test");

    match fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            Ok(true)
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                Ok(false)
            } else {
                Err(format!("Failed to check permissions: {}", e))
            }
        }
    }
}

/// Check if the path is on a mounted filesystem
pub fn check_mount_status(path: &str) -> Result<bool, String> {
    let path_obj = Path::new(path);

    // Try to canonicalize the path
    match path_obj.canonicalize() {
        Ok(canonical) => {
            // If we can read the metadata, the mount is accessible
            match canonical.metadata() {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        Ok(false)
                    } else {
                        Err(format!("Failed to check mount status: {}", e))
                    }
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(false)
            } else {
                Err(format!("Failed to canonicalize path: {}", e))
            }
        }
    }
}
