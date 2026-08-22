use super::model::CueTime;
use super::parse::CueParser;

#[test]
fn test_cue_time_parse() {
    let time = CueTime::parse("03:45:22").unwrap();
    assert_eq!(time.minutes, 3);
    assert_eq!(time.seconds, 45);
    assert_eq!(time.frames, 22);

    let secs = time.to_seconds();
    assert!((secs - 225.293).abs() < 0.01);
}

#[test]
fn test_extract_quoted() {
    assert_eq!(
        CueParser::extract_quoted("TITLE \"My Song\""),
        Some("My Song".to_string())
    );
    assert_eq!(
        CueParser::extract_quoted("FILE \"album.flac\" WAVE"),
        Some("album.flac".to_string())
    );
}

#[test]
fn test_extract_track_number() {
    assert_eq!(CueParser::extract_track_number("TRACK 01 AUDIO"), Some(1));
    assert_eq!(CueParser::extract_track_number("TRACK 12 AUDIO"), Some(12));
}
