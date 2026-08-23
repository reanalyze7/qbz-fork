//! Fetch + map the full label landing page.

use std::collections::HashSet;
use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::Track;

use super::load_sections::{critics_from_page, more_labels_carousel, releases_carousel};
use super::parse::{parse_artist, parse_playlist};
use super::parse_track::parse_top_track;
use super::value_helpers::truncate_words;
use super::LabelPagePayload;
use crate::label::{bl_snapshots, extract_label_image};

/// Fetch + map the full label landing page.
pub async fn load_label_page<A>(
    runtime: &Arc<AppRuntime<A>>,
    label_id: u64,
    fallback_name: &str,
) -> Result<LabelPagePayload, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let page = runtime
        .core()
        .get_label_page(label_id)
        .await
        .map_err(|e| e.to_string())?;

    // Favorite-label ids — seed the header + per-card follow state.
    let follow_ids = favorite_label_ids(runtime).await;

    let name = if page.name.is_empty() {
        fallback_name.to_string()
    } else {
        page.name.clone()
    };
    let image_url = extract_label_image(page.image.as_ref());

    // Description (HTML-stripped) + truncation for the header read-more.
    let description = page
        .description
        .as_deref()
        .map(crate::strip_html::strip_html)
        .unwrap_or_default();
    let description_short = truncate_words(&description, 360);
    let description_truncated = description_short != description;

    // Popular tracks → display rows + play-all queue source.
    let raw_top = page.top_tracks.clone().unwrap_or_default();
    let top_tracks: Vec<super::TopTrack> = raw_top.iter().map(parse_top_track).collect();
    let play_tracks: Vec<Track> = raw_top
        .into_iter()
        .filter_map(|v| serde_json::from_value::<Track>(v).ok())
        .collect();

    let (bl, abl) = bl_snapshots();
    let critics = critics_from_page(&page, &bl, &abl);

    let playlists: Vec<super::PlaylistSlim> = page
        .playlists
        .as_ref()
        .and_then(|p| p.items.as_ref())
        .map(|items| items.iter().map(parse_playlist).collect())
        .unwrap_or_default();

    let artists: Vec<super::ArtistSlim> = page
        .top_artists
        .as_ref()
        .and_then(|a| a.items.as_ref())
        .map(|items| items.iter().map(parse_artist).collect())
        .unwrap_or_default();

    let releases = releases_carousel(runtime, label_id, &bl, &abl).await;
    let more_labels = more_labels_carousel(runtime, label_id, &follow_ids).await;

    Ok(LabelPagePayload {
        id: label_id.to_string(),
        name,
        image_url,
        description,
        description_short,
        description_truncated,
        is_following: follow_ids.contains(&label_id),
        top_tracks,
        releases,
        critics,
        playlists,
        artists,
        more_labels,
        play_tracks,
    })
}

/// The user's favorite-label ids (for the header + more-labels follow
/// state). Best-effort: an error yields an empty set.
async fn favorite_label_ids<A>(runtime: &Arc<AppRuntime<A>>) -> HashSet<u64>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    match runtime.core().get_favorites("label", 500, 0).await {
        Ok(v) => v
            .get("labels")
            .and_then(|l| l.get("items"))
            .and_then(|i| i.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|it| it.get("id").and_then(|x| x.as_u64()))
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => HashSet::new(),
    }
}
