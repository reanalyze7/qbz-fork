//! Album resolution: UPC match if known, else fuzzy text.

use qbz_models::Album;

use crate::cache::CacheLookup;
use crate::matching::{normalize, similarity};
use crate::types::{AlbumCandidate, AlbumReco, RecoSource};
use crate::RecoCatalog;

use super::album_filters::album_if_full;
use super::Cache;

pub use super::album_filters::{is_full_album, is_slop};

const ALBUM_MIN_SCORE: f32 = 0.6;

fn album_cache_key(c: &AlbumCandidate) -> String {
    if let Some(upc) = c.upc.as_deref().filter(|s| !s.is_empty()) {
        format!("alb:upc:{}", upc)
    } else {
        format!("alb:{}|{}", normalize(&c.artist), normalize(&c.title))
    }
}

pub fn build_album_reco(album: &Album, subtitle: String, source: RecoSource) -> AlbumReco {
    let year = album
        .release_date_original
        .as_deref()
        .and_then(|s| s.get(..4).map(|y| y.to_string()))
        .unwrap_or_default();
    let quality_tier = match album.maximum_bit_depth {
        Some(d) if d >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    }
    .to_string();
    let quality_label = match (album.maximum_bit_depth, album.maximum_sampling_rate) {
        (Some(bd), Some(sr)) => format!("{}-bit / {} kHz", bd, sr),
        _ => String::new(),
    };
    AlbumReco {
        qobuz_album_id: album.id.clone(),
        title: album.title.clone(),
        artist: album.artist.name.clone(),
        artist_id: album.artist.id.to_string(),
        year,
        quality_tier,
        quality_label,
        artwork_url: album.image.best().cloned().unwrap_or_default(),
        subtitle,
        source,
    }
}

async fn resolve_album_live(catalog: &dyn RecoCatalog, cand: &AlbumCandidate) -> Option<AlbumReco> {
    if let Some(upc) = cand.upc.as_deref().filter(|s| !s.is_empty()) {
        let albums = catalog.search_albums(upc, 5).await;
        if let Some(a) = albums
            .iter()
            .find(|a| a.upc.as_deref().map(|u| u.eq_ignore_ascii_case(upc)).unwrap_or(false))
        {
            // The UPC pins the exact release; if it's a single/slop, discard the
            // candidate rather than fuzzy-hunting for a different album.
            return album_if_full(a, cand);
        }
    }
    let query = format!("{} {}", cand.artist, cand.title);
    let albums = catalog.search_albums(query.trim(), 10).await;
    let mut best: Option<&Album> = None;
    let mut best_score = 0.0f32;
    for a in &albums {
        let title_s = similarity(&cand.title, &a.title);
        let artist_s = similarity(&cand.artist, &a.artist.name);
        let score = title_s * 0.6 + artist_s * 0.4;
        if score > best_score {
            best_score = score;
            best = Some(a);
        }
    }
    match best {
        Some(a) if best_score >= ALBUM_MIN_SCORE => album_if_full(a, cand),
        _ => None,
    }
}

pub async fn validate_album(
    catalog: &dyn RecoCatalog,
    cache: Cache<'_>,
    cand: &AlbumCandidate,
) -> Option<AlbumReco> {
    let key = album_cache_key(cand);
    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match guard.get(&key) {
                CacheLookup::Found(json) => {
                    if let Ok(mut reco) = serde_json::from_str::<AlbumReco>(&json) {
                        reco.source = cand.source;
                        reco.subtitle = cand.subtitle.clone();
                        return Some(reco);
                    }
                }
                CacheLookup::Negative => return None,
                CacheLookup::Miss => {}
            }
        }
    }
    let reco = resolve_album_live(catalog, cand).await;
    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match &reco {
                Some(r) => guard.put(&key, "album", Some(&serde_json::to_string(r).unwrap_or_default())),
                None => guard.put(&key, "album", None),
            }
        }
    }
    reco
}
