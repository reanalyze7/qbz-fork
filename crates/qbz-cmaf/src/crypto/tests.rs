use super::content_key::{decrypt_frame, unwrap_content_key};
use super::session_key::{derive_session_key, hex_decode};
use super::signature::compute_request_sig;

const TEST_SEED: &str = "00112233445566778899aabbccddeeff";

#[test]
fn test_compute_request_sig() {
    let mut args = std::collections::BTreeMap::new();
    args.insert("profile", "qbz-1".to_string());
    let sig = compute_request_sig("sessionstart", &args, "1775500000", TEST_SEED);
    assert_eq!(sig.len(), 32);
    let sig2 = compute_request_sig("sessionstart", &args, "1775500000", TEST_SEED);
    assert_eq!(sig, sig2);
}

#[test]
fn test_decrypt_frame_roundtrip() {
    let key = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let iv = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let original = b"Hello FLAC frame data here!".to_vec();
    let mut data = original.clone();
    decrypt_frame(&key, &iv, &mut data);
    assert_ne!(data, original);
    decrypt_frame(&key, &iv, &mut data);
    assert_eq!(data, original);
}

#[test]
fn test_derive_session_key_invalid_infos() {
    let result = derive_session_key(TEST_SEED, "no_dot_here");
    assert!(result.is_err());
}

#[test]
fn test_hex_decode_rejects_odd_length() {
    assert!(hex_decode("abc").is_err());
}

#[test]
fn test_hex_decode_rejects_non_hex() {
    assert!(hex_decode("zz").is_err());
    assert!(hex_decode("0g").is_err());
}

#[test]
fn test_hex_decode_empty_and_valid() {
    assert_eq!(hex_decode("").unwrap(), Vec::<u8>::new());
    assert_eq!(hex_decode("00ff").unwrap(), vec![0x00, 0xff]);
    assert_eq!(hex_decode(TEST_SEED).unwrap().len(), 16);
}

#[test]
fn test_derive_session_key_rejects_bad_seed_hex() {
    assert!(derive_session_key("not-hex!!", "YQ.YQ").is_err());
    assert!(derive_session_key("abc", "YQ.YQ").is_err());
}

#[test]
fn test_unwrap_content_key_invalid_format() {
    let key = [0u8; 16];
    let result = unwrap_content_key(&key, "only.two");
    assert!(result.is_err());
}
