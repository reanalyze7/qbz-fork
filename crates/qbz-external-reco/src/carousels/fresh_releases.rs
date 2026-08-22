//! Fresh Releases (ListenBrainz, from artists you follow).

use crate::types::{AlbumCandidate, AlbumReco, RecoSource};
use crate::validate::is_slop;
use crate::RecoInputs;

use super::validate_pools::validate_album_pool;
use super::{rotate_take, DISPLAY_CAP};

pub async fn build_fresh_releases(inputs: &RecoInputs<'_>) -> Vec<AlbumReco> {
    let Some(lb) = &inputs.listenbrainz else {
        return Vec::new();
    };
    let releases = lb.client.get_fresh_releases(&lb.username, 30).await.unwrap_or_default();
    let candidates: Vec<AlbumCandidate> = releases
        .into_iter()
        .filter(|r| {
            !r.release_name.is_empty()
                && !r.artist_credit_name.is_empty()
                && !is_slop(&r.artist_credit_name, &r.release_name)
        })
        .take(50)
        .map(|r| AlbumCandidate {
            artist: r.artist_credit_name,
            title: r.release_name,
            upc: None,
            source: RecoSource::ListenBrainz,
            score: 0.0,
            subtitle: r
                .release_date
                .map(|d| format!("New release · {}", d))
                .unwrap_or_else(|| "New release".to_string()),
        })
        .collect();
    let pool = validate_album_pool(inputs.catalog, inputs.cache, candidates).await;
    rotate_take(pool, inputs.rotation_seed, DISPLAY_CAP)
}
