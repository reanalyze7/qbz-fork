//! Async MusicBrainz-facing load functions for the MusicianPageView.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_integrations::musicbrainz::MusicianConfidence;

use super::PAGE_SIZE;

/// Resolved-musician metadata + the current page bookkeeping. Plain
/// `Send` so the load step can run on a worker.
pub struct MusicianData {
    pub name: String,
    pub role: String,
    pub confidence: MusicianConfidence,
    pub appearances: Vec<AppearanceData>,
    pub total: usize,
}

#[derive(Clone)]
pub struct AppearanceData {
    pub album_id: String,
    pub album_title: String,
    pub artist_name: String,
    pub year: String,
    pub role_on_album: String,
    pub artwork_url: String,
}

/// Resolve the musician + fetch the first page of appearances.
pub async fn load_musician<A>(
    runtime: &Arc<AppRuntime<A>>,
    name: &str,
    role: &str,
) -> Result<MusicianData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let resolved = runtime
        .core()
        .musicbrainz_resolve_musician(name, role)
        .await
        .map_err(|e| e.to_string())?;

    let page = runtime
        .core()
        .musicbrainz_get_musician_appearances(name, role, PAGE_SIZE, 0)
        .await
        .map_err(|e| e.to_string())?;

    let appearances: Vec<AppearanceData> = page
        .albums
        .into_iter()
        .map(|a| AppearanceData {
            album_id: a.album_id,
            album_title: a.album_title,
            artist_name: a.artist_name,
            year: a.year.unwrap_or_default(),
            role_on_album: a.role_on_album,
            artwork_url: a.album_artwork,
        })
        .collect();

    Ok(MusicianData {
        name: resolved.name,
        role: resolved.role,
        confidence: resolved.confidence,
        appearances,
        total: page.total,
    })
}

/// Fetch one more page of appearances, append to the current list.
pub async fn load_more_appearances<A>(
    runtime: &Arc<AppRuntime<A>>,
    name: &str,
    role: &str,
    offset: u32,
) -> Result<(Vec<AppearanceData>, usize), String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let page = runtime
        .core()
        .musicbrainz_get_musician_appearances(name, role, PAGE_SIZE, offset)
        .await
        .map_err(|e| e.to_string())?;
    let appearances = page
        .albums
        .into_iter()
        .map(|a| AppearanceData {
            album_id: a.album_id,
            album_title: a.album_title,
            artist_name: a.artist_name,
            year: a.year.unwrap_or_default(),
            role_on_album: a.role_on_album,
            artwork_url: a.album_artwork,
        })
        .collect();
    Ok((appearances, page.total))
}
