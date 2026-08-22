//! DSF-format integration tests: parsing, PCM conversion, tags, channel
//! validation, and multichannel downmix.

use super::fixtures::write_dsf;
use qbz_dsd::{open_dsd, DsdError, DsdPcmConverter};

#[test]
fn dsf_parses_and_converts_to_exact_frame_count() {
    let path = write_dsf("silence64.dsf", 2, 2_822_400, 2, None);
    let demux = open_dsd(&path).unwrap();
    let info = demux.info().clone();
    assert_eq!(info.dsd_rate, 2_822_400);
    assert_eq!(info.channels, 2);
    assert_eq!(info.sample_count, 2 * 4096 * 8);
    assert!(info.lsb_first);

    let mut conv = DsdPcmConverter::new(demux, -6.0).unwrap();
    let expected_frames = info.sample_count / 32; // DSD64 → 88.2k
    assert_eq!(conv.total_frames(), expected_frames);
    let mut frames = 0u64;
    let mut peak = 0f32;
    while let Some(block) = conv.next_block().unwrap() {
        frames += (block.len() / 2) as u64;
        for s in block {
            peak = peak.max(s.abs());
        }
    }
    assert_eq!(frames, expected_frames);
    assert!(peak < 0.01, "silence converted loud: peak {peak}");
}

#[test]
fn dsf_reads_embedded_id3() {
    let mut tag = id3::Tag::new();
    use id3::TagLike;
    tag.set_title("Karma Police");
    tag.set_artist("Radiohead");
    tag.set_album("OK Computer");
    tag.set_track(6);
    let mut blob = Vec::new();
    tag.write_to(&mut blob, id3::Version::Id3v24).unwrap();

    let path = write_dsf("tagged.dsf", 2, 2_822_400, 1, Some(&blob));
    let demux = open_dsd(&path).unwrap();
    let tags = &demux.info().tags;
    assert_eq!(tags.title.as_deref(), Some("Karma Police"));
    assert_eq!(tags.artist.as_deref(), Some("Radiohead"));
    assert_eq!(tags.album.as_deref(), Some("OK Computer"));
    assert_eq!(tags.track_number, Some(6));
}

#[test]
fn dsf_dsd128_uses_three_stages() {
    let path = write_dsf("silence128.dsf", 2, 5_644_800, 2, None);
    let demux = open_dsd(&path).unwrap();
    let mut conv = DsdPcmConverter::new(demux, -6.0).unwrap();
    assert_eq!(conv.total_frames(), (2 * 4096 * 8) / 64);
    let mut frames = 0u64;
    while let Some(block) = conv.next_block().unwrap() {
        frames += (block.len() / 2) as u64;
    }
    assert_eq!(frames, (2 * 4096 * 8) / 64);
}

#[test]
fn dsf_eight_channels_rejected() {
    let path = write_dsf("multi8.dsf", 8, 2_822_400, 1, None);
    match open_dsd(&path) {
        Err(DsdError::UnsupportedChannels(8)) => {}
        Err(other) => panic!("expected UnsupportedChannels, got {other:?}"),
        Ok(_) => panic!("expected UnsupportedChannels, got Ok"),
    }
}

#[test]
fn dsf_5_1_downmixes_to_stereo() {
    let path = write_dsf("multi51.dsf", 6, 2_822_400, 1, None);
    let demux = open_dsd(&path).unwrap();
    assert_eq!(demux.info().channels, 6);
    let mut conv = DsdPcmConverter::new(demux, -6.0).unwrap();
    assert_eq!(conv.channels(), 2);
    let expected_frames = demux_total_frames(4096 * 8);
    assert_eq!(conv.total_frames(), expected_frames);
    let mut frames = 0u64;
    let mut peak = 0f32;
    while let Some(block) = conv.next_block().unwrap() {
        frames += (block.len() / 2) as u64;
        for s in block {
            peak = peak.max(s.abs());
        }
    }
    assert_eq!(frames, expected_frames);
    assert!(peak < 0.01, "5.1 silence downmix not silent: peak {peak}");
}

fn demux_total_frames(sample_count: u64) -> u64 {
    sample_count / 32 // DSD64 → 88.2 kHz
}
