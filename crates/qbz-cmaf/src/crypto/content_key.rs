use aes::cipher::{BlockDecryptMut, KeyIvInit, StreamCipher};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::error::CmafError;

use super::{Aes128CbcDec, Aes128Ctr};

/// Unwrap the per-track content key using the session key.
///
/// `key_str` format: `"qbz-1.wrapped_key_b64url.iv_b64url"`
pub fn unwrap_content_key(session_key: &[u8; 16], key_str: &str) -> Result<[u8; 16], CmafError> {
    let parts: Vec<&str> = key_str.split('.').collect();
    if parts.len() < 3 {
        return Err(CmafError::InvalidKey(
            "key string must have at least 3 dot-separated parts".into(),
        ));
    }

    let wrapped = URL_SAFE_NO_PAD.decode(parts[1])?;
    let iv = URL_SAFE_NO_PAD.decode(parts[2])?;

    if iv.len() != 16 {
        return Err(CmafError::InvalidKey(format!(
            "unwrap IV must be 16 bytes, got {}",
            iv.len()
        )));
    }

    let mut buf = wrapped.clone();
    let decrypted =
        Aes128CbcDec::new(session_key.into(), iv.as_slice().into())
            .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buf)
            .map_err(|e| CmafError::AesDecrypt(format!("AES-CBC unwrap failed: {e}")))?;

    if decrypted.len() != 16 {
        return Err(CmafError::InvalidKey(format!(
            "unwrapped key must be 16 bytes, got {}",
            decrypted.len()
        )));
    }

    let mut key = [0u8; 16];
    key.copy_from_slice(decrypted);
    Ok(key)
}

/// Decrypt a FLAC frame in-place using AES-128-CTR.
///
/// `iv_8` = 8-byte IV from the segment UUID box entry, zero-padded to 16 bytes.
pub fn decrypt_frame(content_key: &[u8; 16], iv_8: &[u8; 8], data: &mut [u8]) {
    let mut nonce = [0u8; 16];
    nonce[..8].copy_from_slice(iv_8);
    Aes128Ctr::new(content_key.into(), &nonce.into()).apply_keystream(data);
}
