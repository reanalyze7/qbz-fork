//! Pure filesystem-safe filename sanitizing — no I/O.

/// Sanitize filename to be ASCII-safe and filesystem-compatible
pub fn sanitize_filename(name: &str) -> String {
    // Remove or replace invalid characters
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

    let mut sanitized = name
        .chars()
        .map(|c| {
            if invalid_chars.contains(&c) {
                '-'
            } else if c.is_ascii() || c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();

    // Replace multiple consecutive dashes with single dash
    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }

    // Trim dashes and whitespace from ends
    sanitized = sanitized.trim_matches('-').trim().to_string();

    // Limit length to 200 chars (leaving room for extension and path)
    if sanitized.len() > 200 {
        sanitized.truncate(200);
        sanitized = sanitized.trim_matches('-').trim().to_string();
    }

    // If empty after sanitization, use fallback
    if sanitized.is_empty() {
        sanitized = "track".to_string();
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn replaces_invalid_chars_with_dash() {
        assert_eq!(sanitize_filename("AC/DC: Back?"), "AC-DC- Back");
    }

    #[test]
    fn collapses_consecutive_dashes() {
        assert_eq!(sanitize_filename("a///b"), "a-b");
    }

    #[test]
    fn trims_dashes_and_whitespace_from_ends() {
        assert_eq!(sanitize_filename("  -hello-  "), "hello");
    }

    #[test]
    fn falls_back_to_track_when_empty_after_sanitizing() {
        assert_eq!(sanitize_filename("///"), "track");
    }

    #[test]
    fn truncates_to_200_chars() {
        let long_name = "a".repeat(250);
        let result = sanitize_filename(&long_name);
        assert!(result.len() <= 200);
    }

    #[test]
    fn leaves_plain_ascii_name_untouched() {
        assert_eq!(sanitize_filename("Normal Title"), "Normal Title");
    }
}
