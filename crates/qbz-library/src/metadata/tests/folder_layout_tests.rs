use crate::MetadataExtractor;
use std::path::{Path, PathBuf};

#[test]
fn test_album_root_dir_plain() {
    // artist/album/track.flac -> album/
    let path = Path::new("/music/EELS/Beautiful Freak/01 - Novocaine.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/EELS/Beautiful Freak"));
}

#[test]
fn test_album_root_dir_disc_folder() {
    // artist/album/disc1/track.flac -> album/
    let path = Path::new("/music/EELS/Beautiful Freak/Disc 1/01 - Novocaine.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/EELS/Beautiful Freak"));
}

#[test]
fn test_album_root_dir_encoding_folder() {
    // artist/album/quality/track.flac -> album/
    let path = Path::new("/music/EELS/Beautiful Freak/FLAC 24-bit - 96 kHz/01 - Novocaine.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/EELS/Beautiful Freak"));
}

#[test]
fn test_album_root_dir_encoding_and_disc() {
    // artist/album/quality/disc1/track.flac -> album/
    let path =
        Path::new("/music/EELS/Beautiful Freak/FLAC 24-bit - 96 kHz/Disc 1/01 - Novocaine.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/EELS/Beautiful Freak"));
}

#[test]
fn test_infer_artist_album_root_clamp() {
    let roots = vec![PathBuf::from("/music")];

    // Album dir directly under the library root: the root's own name
    // must NOT become the artist (the untagged DSD-at-root case,
    // spec 2026-07-19-local-album-grouping-mode §C).
    let path = Path::new("/music/Some DSD Album/01 - Track.dsf");
    let (artist, album) = MetadataExtractor::infer_artist_album(path, &roots);
    assert_eq!(artist, None);
    assert_eq!(album.as_deref(), Some("Some DSD Album"));

    // Same, behind a disc folder (the clamp looks at the album ROOT dir).
    let path = Path::new("/music/Some DSD Album/Disc 1/01 - Track.dsf");
    let (artist, album) = MetadataExtractor::infer_artist_album(path, &roots);
    assert_eq!(artist, None);
    assert_eq!(album.as_deref(), Some("Some DSD Album"));

    // The "Artist - Album" split still kicks in when the parent-folder
    // inference is clamped away.
    let path = Path::new("/music/MAKE-UP - Saint Seiya Best/01.dsf");
    let (artist, album) = MetadataExtractor::infer_artist_album(path, &roots);
    assert_eq!(artist.as_deref(), Some("MAKE-UP"));
    assert_eq!(album.as_deref(), Some("Saint Seiya Best"));

    // Normal Artist/Album layout is NOT clamped (parent != root).
    let path = Path::new("/music/EELS/Beautiful Freak/01.flac");
    let (artist, album) = MetadataExtractor::infer_artist_album(path, &roots);
    assert_eq!(artist.as_deref(), Some("EELS"));
    assert_eq!(album.as_deref(), Some("Beautiful Freak"));

    // A REAL artist folder whose name matches the root's name is also
    // not clamped (its parent is the root's artist folder, not the root).
    let roots = vec![PathBuf::from("/media/Music")];
    let path = Path::new("/media/Music/Music/Some Album/01.flac");
    let (artist, _) = MetadataExtractor::infer_artist_album(path, &roots);
    assert_eq!(artist.as_deref(), Some("Music"));

    // No roots passed (ephemeral / single-file legacy path): inference
    // is unchanged — parent folder name wins as before.
    let path = Path::new("/music/Some DSD Album/01 - Track.dsf");
    let (artist, _) = MetadataExtractor::infer_artist_album(path, &[]);
    assert_eq!(artist.as_deref(), Some("music"));
}

#[test]
fn test_album_root_dir_album_with_disc_in_name() {
    // Issue #147: album names containing "Disc N" should NOT be treated as disc folders
    let path = Path::new("/music/Various Artists/100 Popular Classics, Disc 1/01 - Track.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(
        root,
        Path::new("/music/Various Artists/100 Popular Classics, Disc 1")
    );

    let path = Path::new("/music/Various Artists/Relaxation Disc1/01 - Track.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/Various Artists/Relaxation Disc1"));

    let path = Path::new("/music/Various Artists/Now 75 - CD1/01 - Track.flac");
    let root = MetadataExtractor::album_root_dir(path).unwrap();
    assert_eq!(root, Path::new("/music/Various Artists/Now 75 - CD1"));
}
