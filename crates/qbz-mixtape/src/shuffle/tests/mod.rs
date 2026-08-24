use qbz_models::QueueTrack as CoreQueueTrack;
use rand::SeedableRng;

mod dedup_similarity_tests;
mod dedup_tests;
mod normalize_tests;
mod sample_tests;

pub(super) fn mk_track(id: u64, title: &str, artist: &str, album_id: Option<&str>) -> CoreQueueTrack {
    CoreQueueTrack {
        id,
        title: title.to_string(),
        version: None,
        artist: artist.to_string(),
        album: String::new(),
        album_version: None,
        duration_secs: 0,
        artwork_url: None,
        hires: false,
        bit_depth: None,
        sample_rate: None,
        is_local: false,
        album_id: album_id.map(|s| s.to_string()),
        artist_id: None,
        streamable: true,
        source: None,
        parental_warning: false,
        source_item_id_hint: None,
        context_kind: None,
        context_id: None,
    }
}

pub(super) fn deterministic_rng() -> rand::rngs::StdRng {
    rand::rngs::StdRng::seed_from_u64(42)
}
