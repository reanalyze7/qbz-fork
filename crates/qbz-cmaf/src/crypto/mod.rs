//! CMAF/FLAC decryption crypto primitives for Qobuz streaming: HKDF
//! session-key derivation, AES-CBC content-key unwrapping, AES-CTR frame
//! decryption, and the MD5 request-signature helper used by the CMAF
//! session/start API calls.

mod content_key;
mod session_key;
mod signature;

#[cfg(test)]
mod tests;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128Ctr = ctr::Ctr128BE<aes::Aes128>;

pub use content_key::{decrypt_frame, unwrap_content_key};
pub use session_key::derive_session_key;
pub use signature::compute_request_sig;
