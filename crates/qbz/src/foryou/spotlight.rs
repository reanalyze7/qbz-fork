//! Spotlight section: a rotated favorite artist's page.

use std::collections::HashSet;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Artist;

use super::{AlbumCard, SpotlightData};

pub(super) async fn load_spotlight<A>(
    runtime: &Arc<AppRuntime<A>>,
    favorites: &[Artist],
) -> Option<SpotlightData>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if favorites.is_empty() {
        return None;
    }
    // Rotate among the top 5 favorites by wall-clock seconds.
    let pool = favorites.len().min(5);
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize % pool)
        .unwrap_or(0);
    let seed = &favorites[idx];

    let page = runtime.core().get_artist_page(seed.id, None).await.ok()?;
    let image_url = page
        .images
        .as_ref()
        .and_then(|i| i.portrait.as_ref())
        .map(|p| {
            format!(
                "https://static.qobuz.com/images/artists/covers/medium/{}.{}",
                p.hash, p.format
            )
        })
        .unwrap_or_default();

    // Up to 6 albums, preferring full albums then live/ep/compilation.
    let mut seen: HashSet<String> = HashSet::new();
    let mut albums: Vec<AlbumCard> = Vec::new();
    for want in ["album", "live", "ep-single", "compilation"] {
        if albums.len() >= 6 {
            break;
        }
        let Some(groups) = page.releases.as_ref() else {
            break;
        };
        let Some(group) = groups.iter().find(|g| g.release_type == want) else {
            continue;
        };
        for rel in &group.items {
            if !seen.insert(rel.id.clone()) {
                continue;
            }
            // Drop blocked albums (own id) and blacklisted-artist releases.
            let rel_artist_id = rel
                .artist
                .as_ref()
                .map(|a| a.id.to_string())
                .unwrap_or_default();
            if crate::artist_blacklist::card_blacklisted(&rel.id, &rel_artist_id) {
                continue;
            }
            let year = rel
                .dates
                .as_ref()
                .and_then(|d| d.original.as_deref())
                .and_then(|s| s.get(..4).map(|y| y.to_string()))
                .unwrap_or_default();
            let bd = rel.audio_info.as_ref().and_then(|a| a.maximum_bit_depth);
            let sr = rel.audio_info.as_ref().and_then(|a| a.maximum_sampling_rate);
            albums.push(AlbumCard {
                id: rel.id.clone(),
                title: rel.title.clone(),
                artist: rel
                    .artist
                    .as_ref()
                    .map(|a| a.name.display.clone())
                    .unwrap_or_else(|| page.name.display.clone()),
                artist_id: rel
                    .artist
                    .as_ref()
                    .map(|a| a.id.to_string())
                    .unwrap_or_default(),
                year,
                quality_tier: match bd {
                    Some(d) if d >= 24 => "hires",
                    Some(_) => "cd",
                    None => "",
                }
                .to_string(),
                quality_label: match (bd, sr) {
                    (Some(b), Some(r)) => format!("{}-bit / {} kHz", b, r),
                    _ => String::new(),
                },
                artwork_url: rel
                    .image
                    .as_ref()
                    .and_then(|img| img.best().cloned())
                    .unwrap_or_default(),
            });
            if albums.len() >= 6 {
                break;
            }
        }
    }

    Some(SpotlightData {
        artist_id: seed.id.to_string(),
        artist_name: page.name.display.clone(),
        category: page.artist_category.clone().unwrap_or_default(),
        image_url,
        has_top_tracks: page.top_tracks.as_ref().map(|t| !t.is_empty()).unwrap_or(false),
        albums,
    })
}
