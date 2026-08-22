//! Error propagation and blocking-read tests for `BufferedMediaSource` +
//! `BufferWriter`.

use std::io::{ErrorKind, Read};
use std::thread;
use std::time::Duration;

use crate::player::streaming_source::{BufferedMediaSource, StreamingConfig};

#[test]
fn first_error_wins_over_later_generic_abort() {
    let (source, writer) = BufferedMediaSource::new(StreamingConfig::from_seconds(1), None);
    writer.error("root cause".to_string()).unwrap();
    writer
        .error("CMAF stream aborted before completion".to_string())
        .unwrap();
    assert_eq!(source.download_error().as_deref(), Some("root cause"));
}

#[test]
fn feeder_error_is_visible_to_waiters_before_min_buffer() {
    let (source, writer) = BufferedMediaSource::new(StreamingConfig::from_seconds(1), None);
    assert!(!source.has_min_buffer());
    assert!(source.download_error().is_none());
    writer.error("feeder died".to_string()).unwrap();
    // The initial-buffer wait loop polls this instead of sleeping out
    // the full buffer timeout.
    assert_eq!(source.download_error().as_deref(), Some("feeder died"));
    assert!(!source.has_min_buffer());
}

#[test]
fn test_blocking_read() {
    let config = StreamingConfig {
        initial_buffer_bytes: 5,
        max_buffer_bytes: 100,
    };
    let (mut source, writer) = BufferedMediaSource::new(config, None);

    // Spawn thread to write after delay
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        writer.push_chunk(b"Delayed").unwrap();
        writer.complete().unwrap();
    });

    // This should block until data arrives
    let mut buf = [0u8; 7];
    let n = source.read(&mut buf).unwrap();
    assert_eq!(n, 7);
    assert_eq!(&buf, b"Delayed");
}

#[test]
fn error_unblocks_reader_with_io_error() {
    let config = StreamingConfig {
        initial_buffer_bytes: 5,
        max_buffer_bytes: 100,
    };
    let (mut source, writer) = BufferedMediaSource::new(config, None);
    writer.error("cdn failed".into()).unwrap();
    let mut buf = [0u8; 8];
    let err = source.read(&mut buf).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Other);
    let msg = err.to_string();
    assert!(msg.contains("cdn failed"), "{msg}");
}
