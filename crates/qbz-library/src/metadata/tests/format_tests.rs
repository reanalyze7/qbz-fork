use crate::{AudioFormat, MetadataExtractor};
use std::path::Path;

#[test]
fn test_detect_format() {
    assert_eq!(
        MetadataExtractor::detect_format(Path::new("test.flac")),
        AudioFormat::Flac
    );
    assert_eq!(
        MetadataExtractor::detect_format(Path::new("test.m4a")),
        AudioFormat::Alac
    );
    assert_eq!(
        MetadataExtractor::detect_format(Path::new("test.wav")),
        AudioFormat::Wav
    );
    assert_eq!(
        MetadataExtractor::detect_format(Path::new("test.mp3")),
        AudioFormat::Mp3
    );
}

#[test]
fn test_is_encoding_folder() {
    // QBZ-generated quality folder names
    assert!(MetadataExtractor::is_encoding_folder(
        "FLAC 16-bit - 44.1 kHz"
    ));
    assert!(MetadataExtractor::is_encoding_folder(
        "FLAC 24-bit - 96 kHz"
    ));
    assert!(MetadataExtractor::is_encoding_folder(
        "FLAC 24-bit - 192 kHz"
    ));
    assert!(MetadataExtractor::is_encoding_folder("MP3 320 kbps"));

    // Common encoding folder names from other tools
    assert!(MetadataExtractor::is_encoding_folder("FLAC"));
    assert!(MetadataExtractor::is_encoding_folder("flac"));
    assert!(MetadataExtractor::is_encoding_folder("MP3"));
    assert!(MetadataExtractor::is_encoding_folder("WAV"));
    assert!(MetadataExtractor::is_encoding_folder("ALAC"));
    assert!(MetadataExtractor::is_encoding_folder("DSD"));
    assert!(MetadataExtractor::is_encoding_folder("320kbps"));

    // Not encoding folders
    assert!(!MetadataExtractor::is_encoding_folder("Abbey Road"));
    assert!(!MetadataExtractor::is_encoding_folder("Disc 1"));
    assert!(!MetadataExtractor::is_encoding_folder("The Beatles"));
    assert!(!MetadataExtractor::is_encoding_folder("2024"));
}
