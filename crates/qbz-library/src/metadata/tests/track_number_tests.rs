use crate::MetadataExtractor;
use std::path::Path;

#[test]
fn test_infer_track_number_from_filename() {
    // Common patterns: "01 - Title"
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new(
            "/music/01 - Novocaine.flac"
        )),
        Some(1)
    );
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new(
            "/music/12 - Beautiful Freak.flac"
        )),
        Some(12)
    );
    // "01. Title"
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new(
            "/music/03. Song Name.flac"
        )),
        Some(3)
    );
    // "01_Title"
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new("/music/05_Track.flac")),
        Some(5)
    );
    // Just number
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new("/music/07.flac")),
        Some(7)
    );
    // "Track 01"
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new("/music/Track 09.flac")),
        Some(9)
    );
    // "1-01 Title" (disc-track)
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new("/music/2-05 Song.flac")),
        Some(5)
    );
    // Not a track number (title starting with non-track digits)
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new("/music/Novocaine.flac")),
        None
    );
    // Zero is not a valid track number
    assert_eq!(
        MetadataExtractor::infer_track_number_from_filename(Path::new(
            "/music/00 - Intro.flac"
        )),
        None
    );
}
