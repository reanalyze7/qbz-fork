use md5::{Digest, Md5};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate MD5 signature for protected API endpoints
///
/// Signature format: MD5(method + params + timestamp + secret)
pub fn generate_signature(method: &str, params: &str, timestamp: u64, secret: &str) -> String {
    let sig_string = format!("{}{}{}{}", method, params, timestamp, secret);
    let mut hasher = Md5::new();
    hasher.update(sig_string.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate signature for track/getFileUrl endpoint
pub fn sign_get_file_url(track_id: u64, format_id: u32, timestamp: u64, secret: &str) -> String {
    let params = format!("format_id{}intentstreamtrack_id{}", format_id, track_id);
    generate_signature("trackgetFileUrl", &params, timestamp, secret)
}

/// Generate signature for favorite/getUserFavorites endpoint
pub fn sign_get_favorites(timestamp: u64, secret: &str) -> String {
    generate_signature("favoritegetUserFavorites", "", timestamp, secret)
}

/// Generate signature for search endpoints.
/// `method` is the concatenated endpoint name (e.g. "catalogsearch", "albumsearch").
/// Query params are sorted alphabetically: limit, offset, query, [type].
pub fn sign_search(
    method: &str,
    query: &str,
    limit: u32,
    offset: u32,
    search_type: Option<&str>,
    timestamp: u64,
    secret: &str,
) -> String {
    let mut params = format!("limit{}offset{}query{}", limit, offset, query);
    if let Some(st) = search_type {
        params.push_str(&format!("type{}", st));
    }
    generate_signature(method, &params, timestamp, secret)
}

/// Generic request signature for any endpoint.
///
/// `method` is the endpoint path with slashes removed, e.g. "/album/get" → "albumget".
/// `kv_pairs` are (key, value) pairs sorted alphabetically by key.
/// The params string is built as key1value1key2value2... (same as mobile app interceptor).
pub fn sign_request(method: &str, kv_pairs: &[(&str, &str)], timestamp: u64, secret: &str) -> String {
    let mut sorted = kv_pairs.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let params: String = sorted.iter().map(|(k, v)| format!("{}{}", k, v)).collect();
    generate_signature(method, &params, timestamp, secret)
}

/// Get current Unix timestamp
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
