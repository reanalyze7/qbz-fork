//! Read/write/seek tests for `BufferedMediaSource` + `BufferWriter`.
//! Error/blocking-behavior tests live in `tests_buffer_errors.rs`.

use std::io::{Read, Seek, SeekFrom};

use crate::player::streaming_source::{BufferedMediaSource, StreamingConfig};

#[test]
fn test_basic_read_write() {
    let config = StreamingConfig {
        initial_buffer_bytes: 10,
        max_buffer_bytes: 100,
    };
    let (mut source, writer) = BufferedMediaSource::new(config, Some(20));

    // Write some data
    writer.push_chunk(b"Hello").unwrap();
    writer.push_chunk(b"World").unwrap();

    // Read it back
    let mut buf = [0u8; 5];
    assert_eq!(source.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"Hello");

    assert_eq!(source.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"World");
}

#[test]
fn test_seek_within_buffer() {
    let config = StreamingConfig {
        initial_buffer_bytes: 5,
        max_buffer_bytes: 100,
    };
    let (mut source, writer) = BufferedMediaSource::new(config, Some(10));

    writer.push_chunk(b"0123456789").unwrap();
    writer.complete().unwrap();

    // Read first 5 bytes
    let mut buf = [0u8; 5];
    source.read(&mut buf).unwrap();
    assert_eq!(&buf, b"01234");

    // Seek back to start
    source.seek(SeekFrom::Start(0)).unwrap();
    source.read(&mut buf).unwrap();
    assert_eq!(&buf, b"01234");

    // Seek to middle
    source.seek(SeekFrom::Start(3)).unwrap();
    source.read(&mut buf).unwrap();
    assert_eq!(&buf, b"34567");
}

#[test]
fn test_complete_data_retrieval() {
    let config = StreamingConfig {
        initial_buffer_bytes: 5,
        max_buffer_bytes: 100,
    };
    let (source, writer) = BufferedMediaSource::new(config, Some(10));

    writer.push_chunk(b"Hello").unwrap();
    assert!(source.take_complete_data().is_none()); // Not complete yet

    writer.push_chunk(b"World").unwrap();
    writer.complete().unwrap();

    let data = source.take_complete_data().unwrap();
    assert_eq!(&data, b"HelloWorld");
}
