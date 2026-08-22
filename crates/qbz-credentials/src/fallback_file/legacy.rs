//! Reading the pre-encryption legacy XOR-obfuscated fallback format, used
//! only during one-shot migration to the AES-256-GCM format.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;
use std::path::PathBuf;

use crate::crypto::legacy_deobfuscate;
use crate::QobuzCredentials;

/// Try to load credentials from legacy XOR format
pub(super) fn load_legacy_credentials(path: &PathBuf) -> Result<Option<QobuzCredentials>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let encoded =
        fs::read_to_string(path).map_err(|e| format!("Failed to read legacy file: {}", e))?;

    let obfuscated = BASE64
        .decode(encoded.trim())
        .map_err(|e| format!("Failed to decode legacy data: {}", e))?;

    let json_bytes = legacy_deobfuscate(&obfuscated);
    let json = String::from_utf8(json_bytes)
        .map_err(|e| format!("Failed to decode legacy credentials: {}", e))?;

    serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse legacy credentials: {}", e))
        .map(Some)
}
