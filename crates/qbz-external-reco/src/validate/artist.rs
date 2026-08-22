//! Artist resolution: Qobuz artist-search + normalized-name match.

use crate::cache::CacheLookup;
use crate::matching::normalize;
use crate::types::{ArtistCandidate, ArtistReco};
use crate::RecoCatalog;

use super::Cache;

pub async fn validate_artist(
    catalog: &dyn RecoCatalog,
    cache: Cache<'_>,
    cand: &ArtistCandidate,
) -> Option<ArtistReco> {
    let key = format!("a:{}", normalize(&cand.name));
    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match guard.get(&key) {
                CacheLookup::Found(json) => {
                    if let Ok(mut reco) = serde_json::from_str::<ArtistReco>(&json) {
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

    let target = normalize(&cand.name);
    let artists = catalog.search_artists(&cand.name, 8).await;
    let reco = artists
        .iter()
        .find(|a| normalize(&a.name) == target)
        .or_else(|| artists.first())
        .filter(|a| a.id != 0)
        .map(|a| ArtistReco {
            qobuz_artist_id: a.id,
            name: a.name.clone(),
            image_url: a.image.as_ref().and_then(|i| i.best().cloned()).unwrap_or_default(),
            subtitle: cand.subtitle.clone(),
            source: cand.source,
        });

    if let Some(c) = cache {
        if let Ok(guard) = c.lock() {
            match &reco {
                Some(r) => guard.put(&key, "artist", Some(&serde_json::to_string(r).unwrap_or_default())),
                None => guard.put(&key, "artist", None),
            }
        }
    }
    reco
}
