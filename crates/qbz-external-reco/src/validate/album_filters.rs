//! Full-album/not-slop pre-filters shared by the album resolution pipeline.

use qbz_models::Album;

use crate::types::AlbumCandidate;

use super::build_album_reco;
use crate::types::AlbumReco;

/// Minimum track count to treat a release as a full album when Qobuz did not
/// label its `release_type` (singles/EPs are short).
const MIN_ALBUM_TRACKS: u32 = 5;

/// Keep only proper full albums — drop singles, EPs, boxsets, compilations.
/// Qobuz's `release_type` is the source of truth ("album" | "single" |
/// "boxset" | "compilation"); when it is absent, fall back to the track count.
pub fn is_full_album(album: &Album) -> bool {
    match album.release_type.as_deref() {
        Some(rt) => rt.eq_ignore_ascii_case("album"),
        None => album.tracks_count.or(album.track_count).unwrap_or(0) >= MIN_ALBUM_TRACKS,
    }
}

/// Second layer against karaoke / tribute / "made famous by" AI-slop that can
/// still wear a full-album shape. Matched on the artist OR title, case-folded.
pub fn is_slop(artist: &str, title: &str) -> bool {
    const NEEDLES: &[&str] = &[
        "karaoke",
        "tribute to",
        "tribute band",
        "made famous by",
        "made popular by",
        "as made famous",
        "as made popular",
        "originally performed",
        "in the style of",
        "instrumental version",
    ];
    let a = artist.to_lowercase();
    let t = title.to_lowercase();
    NEEDLES.iter().any(|n| a.contains(n) || t.contains(n))
}

/// Build the reco only if the resolved Qobuz album is a real full album and not
/// karaoke/tribute slop; otherwise discard the candidate (cached as negative).
pub(super) fn album_if_full(a: &Album, cand: &AlbumCandidate) -> Option<AlbumReco> {
    if is_full_album(a) && !is_slop(&a.artist.name, &a.title) {
        Some(build_album_reco(a, cand.subtitle.clone(), cand.source))
    } else {
        None
    }
}
