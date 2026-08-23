use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use qbz_models::lenient::parse_items_lenient as parse_items;
use qbz_models::{Album, Artist, Track};

use super::FavData;
use crate::favorites::mapping::{map_artist, map_label, map_track};
use crate::favorites::{FavTab, MAX_ITEMS, PAGE_SIZE};

use super::FavLabel;

/// Page through the favorites until the API is exhausted (mirrors Tauri's
/// fetchAllFavorites: keep pulling until a short page or offset >= total),
/// capped at MAX_ITEMS so a pathological library can't loop forever. Then
/// parse + map the tab-specific branch.
pub(crate) async fn load_generic<A>(
    runtime: &Arc<AppRuntime<A>>,
    tab: FavTab,
) -> Result<FavData, String>
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let mut total: usize;
    let mut all_items: Vec<serde_json::Value> = Vec::new();
    let mut offset = 0u32;
    loop {
        let value = runtime
            .core()
            .get_favorites(tab.key(), PAGE_SIZE, offset)
            .await
            .map_err(|e| e.to_string())?;
        let branch = value.get(tab.key());
        total = branch
            .and_then(|b| b.get("total"))
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;
        let page: Vec<serde_json::Value> = branch
            .and_then(|b| b.get("items"))
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();
        let page_len = page.len();
        all_items.extend(page);
        offset += page_len as u32;
        let exhausted = page_len < PAGE_SIZE as usize
            || (total > 0 && offset as usize >= total)
            || all_items.len() >= MAX_ITEMS;
        if exhausted {
            break;
        }
    }
    Ok(match tab {
        FavTab::Tracks => {
            let tracks: Vec<Track> = parse_items(all_items, "track");
            let play = tracks.clone();
            FavData::Tracks {
                items: tracks.into_iter().map(map_track).collect(),
                play,
                total,
            }
        }
        FavTab::Albums => {
            let albums: Vec<Album> = parse_items(all_items, "album");
            // Drop blocked albums at the SOURCE so the model + the artwork jobs
            // (both derived from `items`) stay index-aligned.
            let (bl, abl) = if crate::artist_blacklist::is_enabled() {
                (
                    crate::artist_blacklist::ids_snapshot(),
                    crate::artist_blacklist::album_ids_snapshot(),
                )
            } else {
                Default::default()
            };
            FavData::Albums {
                items: albums
                    .into_iter()
                    .filter(|a| !qbz_core::core::album_blacklisted(a, &bl, &abl))
                    .map(crate::album_map::map_album)
                    .collect(),
                total,
            }
        }
        FavTab::Artists => {
            let artists: Vec<Artist> = parse_items(all_items, "artist");
            FavData::Artists {
                items: artists.into_iter().map(map_artist).collect(),
                total,
            }
        }
        FavTab::Labels => {
            let labels: Vec<FavLabel> = parse_items(all_items, "label");
            FavData::Labels {
                items: labels.into_iter().map(map_label).collect(),
                total,
            }
        }
        FavTab::Playlists => unreachable!("handled above"),
    })
}
