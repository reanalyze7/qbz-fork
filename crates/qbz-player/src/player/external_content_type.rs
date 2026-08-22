use super::*;

/// Pick the MIME to advertise to an external renderer for a legacy stream-URL
/// download. Prefer the server-provided `mime_type`; when it is empty (Qobuz
/// can return `""`), fall back by format id so the renderer is never handed an
/// empty content type (which some Chromecast/DLNA renderers reject).
pub fn external_content_type(mime: &str, format_id: u32) -> String {
    let trimmed = mime.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    match qbz_models::Quality::from_id(format_id) {
        Some(qbz_models::Quality::Mp3) => "audio/mpeg".to_string(),
        // Lossless / HiRes / UltraHiRes are FLAC over the file/url path.
        Some(_) => "audio/flac".to_string(),
        None => "audio/flac".to_string(),
    }
}
