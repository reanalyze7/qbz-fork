//! AES-256-GCM encrypt/decrypt of [`QobuzCredentials`], plus the legacy XOR
//! deobfuscation used only when migrating pre-encryption fallback files.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::keys::{derive_key_at, PortalKey};
use crate::paths::config_qbz_root;
use crate::QobuzCredentials;

// Legacy XOR key for migration (only used for reading old format)
const LEGACY_OBFUSCATION_KEY: &[u8] = b"QbzNixAudiophile2024";

/// Encrypted data format stored in file
#[derive(Serialize, Deserialize)]
struct EncryptedCredentials {
    /// Version for future format changes
    version: u8,
    /// Base64-encoded nonce (12 bytes for AES-GCM)
    nonce: String,
    /// Base64-encoded ciphertext
    ciphertext: String,
}

/// Encrypt credentials using AES-256-GCM (desktop root).
pub(crate) fn encrypt_credentials(credentials: &QobuzCredentials) -> Result<String, String> {
    let root = config_qbz_root().ok_or("Could not determine config directory")?;
    encrypt_credentials_at(&root, PortalKey::Session, credentials)
}

/// Encrypt credentials using AES-256-GCM, deriving the key under `root`.
pub(crate) fn encrypt_credentials_at(
    root: &Path,
    portal: PortalKey,
    credentials: &QobuzCredentials,
) -> Result<String, String> {
    let key = derive_key_at(root, portal)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Failed to create cipher: {}", e))?;

    // Generate random nonce
    let mut nonce_raw = [0u8; 12];
    rand::rng().fill(&mut nonce_raw);
    let nonce_bytes: [u8; 12] = aes_gcm::aead::generic_array::GenericArray::from(nonce_raw).into();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let json = serde_json::to_string(credentials)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, json.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let encrypted = EncryptedCredentials {
        version: 1,
        nonce: BASE64.encode(nonce_bytes),
        ciphertext: BASE64.encode(ciphertext),
    };

    serde_json::to_string(&encrypted)
        .map_err(|e| format!("Failed to serialize encrypted data: {}", e))
}

/// Decrypt credentials using AES-256-GCM (desktop root).
pub(crate) fn decrypt_credentials(encrypted_json: &str) -> Result<QobuzCredentials, String> {
    let root = config_qbz_root().ok_or("Could not determine config directory")?;
    decrypt_credentials_at(&root, PortalKey::Session, encrypted_json)
}

/// Decrypt credentials using AES-256-GCM, deriving the key under `root`.
pub(crate) fn decrypt_credentials_at(
    root: &Path,
    portal: PortalKey,
    encrypted_json: &str,
) -> Result<QobuzCredentials, String> {
    let encrypted: EncryptedCredentials = serde_json::from_str(encrypted_json)
        .map_err(|e| format!("Failed to parse encrypted data: {}", e))?;

    if encrypted.version != 1 {
        return Err(format!(
            "Unsupported encryption version: {}",
            encrypted.version
        ));
    }

    let key = derive_key_at(root, portal)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Failed to create cipher: {}", e))?;

    let nonce_bytes = BASE64
        .decode(&encrypted.nonce)
        .map_err(|e| format!("Failed to decode nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = BASE64
        .decode(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Decryption failed (wrong key or corrupted data)".to_string())?;

    let json = String::from_utf8(plaintext)
        .map_err(|e| format!("Failed to decode decrypted data: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse credentials: {}", e))
}

/// Legacy XOR deobfuscation (for migration only)
pub(crate) fn legacy_deobfuscate(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ LEGACY_OBFUSCATION_KEY[i % LEGACY_OBFUSCATION_KEY.len()])
        .collect()
}
