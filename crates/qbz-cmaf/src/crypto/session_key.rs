use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::CmafError;

/// Strict hex decode for the app seed (HKDF IKM). Fail closed on odd length
/// or non-hex digits — never zero-fill bad nibbles.
pub(super) fn hex_decode(hex: &str) -> Result<Vec<u8>, CmafError> {
    if hex.len() % 2 != 0 {
        return Err(CmafError::InvalidInfos(format!(
            "seed hex must have even length, got {}",
            hex.len()
        )));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
            CmafError::InvalidInfos(format!("invalid seed hex digit at offset {i}"))
        })?;
        out.push(byte);
    }
    Ok(out)
}

/// Derive the 16-byte session key from the session/start `infos` field.
///
/// `infos` format: `"salt_b64url.info_b64url"`
/// `seed` is the hex-encoded app seed used as IKM for HKDF (provided by caller).
pub fn derive_session_key(seed: &str, infos: &str) -> Result<[u8; 16], CmafError> {
    let parts: Vec<&str> = infos.split('.').collect();
    if parts.len() < 2 {
        return Err(CmafError::InvalidInfos(
            "session infos must have at least 2 dot-separated parts".into(),
        ));
    }

    let salt = URL_SAFE_NO_PAD.decode(parts[0])?;
    let info = URL_SAFE_NO_PAD.decode(parts[1])?;

    let ikm = hex_decode(seed)?;

    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 16];
    hk.expand(&info, &mut okm).map_err(|_| CmafError::HkdfExpand)?;

    Ok(okm)
}
