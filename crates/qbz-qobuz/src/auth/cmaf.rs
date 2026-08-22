/// App seed for CMAF request signing and key derivation.
/// Public value extracted from the webplayer JS bundle.
pub const CMAF_SEED: &str = "abb21364945c0583309667d13ca3d93a";

/// Generate CMAF signature for session/start endpoint
pub fn sign_session_start(timestamp: u64) -> String {
    let mut args = std::collections::BTreeMap::new();
    args.insert("profile", "qbz-1".to_string());
    qbz_cmaf::compute_request_sig("sessionstart", &args, &timestamp.to_string(), CMAF_SEED)
}

/// Generate CMAF signature for file/url endpoint
pub fn sign_file_url(track_id: u64, format_id: u32, timestamp: u64) -> String {
    let mut args = std::collections::BTreeMap::new();
    args.insert("format_id", format_id.to_string());
    args.insert("intent", "stream".to_string());
    args.insert("track_id", track_id.to_string());
    qbz_cmaf::compute_request_sig("fileurl", &args, &timestamp.to_string(), CMAF_SEED)
}
