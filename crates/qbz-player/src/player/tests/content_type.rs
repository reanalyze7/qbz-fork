use super::*;

#[test]
fn external_content_type_prefers_server_mime() {
    assert_eq!(external_content_type("audio/flac", 7), "audio/flac");
    assert_eq!(external_content_type("audio/mpeg", 5), "audio/mpeg");
    // Whitespace-only is treated as empty.
    assert_eq!(external_content_type("  ", 6), "audio/flac");
}

#[test]
fn external_content_type_falls_back_by_format_id() {
    // Empty MIME (Qobuz can return "") -> derive from format id.
    assert_eq!(external_content_type("", 5), "audio/mpeg"); // Mp3
    assert_eq!(external_content_type("", 6), "audio/flac"); // Lossless
    assert_eq!(external_content_type("", 7), "audio/flac"); // HiRes
    assert_eq!(external_content_type("", 27), "audio/flac"); // UltraHiRes
    assert_eq!(external_content_type("", 999), "audio/flac"); // unknown -> flac
}
