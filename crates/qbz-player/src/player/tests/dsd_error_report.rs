use super::*;

struct FakeDsdSource {
    words: std::vec::IntoIter<i32>,
    error: Option<String>,
}

impl Iterator for FakeDsdSource {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        self.words.next()
    }
}

impl qbz_dsd::DsdWordSource for FakeDsdSource {
    fn io_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[test]
fn dsd_error_report_surfaces_io_error_at_stream_end() {
    let state = SharedState::new();
    let source = FakeDsdSource {
        words: vec![1, 2].into_iter(),
        error: Some("read failed".to_string()),
    };
    let mut wrapped = DsdErrorReport::new(source, state.clone());
    assert_eq!(wrapped.next(), Some(1));
    assert!(!state.has_stream_error());
    assert_eq!(wrapped.next(), Some(2));
    assert_eq!(wrapped.next(), None);
    assert!(state.has_stream_error());
    let msg = state.take_stream_error_message().unwrap();
    assert!(msg.contains("read failed"));
    // Repeated polls after exhaustion must not re-record the message.
    assert_eq!(wrapped.next(), None);
    assert_eq!(state.take_stream_error_message(), None);
}

#[test]
fn dsd_error_report_clean_eof_is_not_an_error() {
    let state = SharedState::new();
    let source = FakeDsdSource {
        words: vec![1].into_iter(),
        error: None,
    };
    let mut wrapped = DsdErrorReport::new(source, state.clone());
    assert_eq!(wrapped.next(), Some(1));
    assert_eq!(wrapped.next(), None);
    assert!(!state.has_stream_error());
    assert_eq!(state.take_stream_error_message(), None);
}
