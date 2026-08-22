//! Scan progress and audio property models

use serde::{Deserialize, Serialize};

/// Scan progress for UI updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub status: ScanStatus,
    pub total_files: u32,
    pub processed_files: u32,
    pub current_file: Option<String>,
    pub errors: Vec<ScanError>,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            status: ScanStatus::Idle,
            total_files: 0,
            processed_files: 0,
            current_file: None,
            errors: Vec::new(),
        }
    }
}

/// Scan status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Complete,
    Cancelled,
    Error,
}

/// A scan error for a specific file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub file_path: String,
    pub error: String,
}

/// Audio properties extracted from a file
#[derive(Debug, Clone, Default)]
pub struct AudioProperties {
    pub duration_secs: u64,
    pub bit_depth: Option<u32>,
    pub sample_rate: f64, // Changed from u32 to f64 for decimal precision
    pub channels: u8,
}
