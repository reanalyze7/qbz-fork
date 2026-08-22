//! Pure validation of a completed download's byte count.

/// Validate downloaded byte count against optional Content-Length.
/// Fail closed when length is known and mismatched, or when zero bytes.
pub(crate) fn validate_download_size(
    track_id: u64,
    cached: u64,
    content_length: Option<u64>,
) -> Result<(), String> {
    if let Some(expected) = content_length {
        if cached != expected {
            return Err(format!(
                "Truncated download for track {track_id}: got {cached} bytes, Content-Length was {expected}"
            ));
        }
    }
    if cached == 0 {
        return Err(format!("Empty download for track {track_id}"));
    }
    Ok(())
}

#[cfg(test)]
mod download_size_tests {
    use super::validate_download_size;

    #[test]
    fn accepts_matching_length() {
        assert!(validate_download_size(1, 100, Some(100)).is_ok());
    }

    #[test]
    fn rejects_truncated() {
        let err = validate_download_size(7, 50, Some(100)).unwrap_err();
        assert!(err.contains("Truncated"));
        assert!(err.contains("50"));
        assert!(err.contains("100"));
    }

    #[test]
    fn rejects_empty_even_without_length() {
        assert!(validate_download_size(2, 0, None).is_err());
    }

    #[test]
    fn accepts_nonzero_unknown_length() {
        assert!(validate_download_size(3, 42, None).is_ok());
    }
}
