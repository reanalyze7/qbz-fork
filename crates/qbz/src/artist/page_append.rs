use std::collections::HashSet;

use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::artist::cache::{FULL_RELEASE_SECTIONS, LOADED_PAGES, MAX_INDEX_PAGES};
use crate::artist::track_map::card_to_item;
use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::home::CardData;
use crate::{AlbumCardItem, AppWindow, ArtistReleaseSection, ArtistState};

/// Append a freshly-fetched page to a bucket (dedupe by id, re-sort, update
/// has_more honoring the 4-page cap). Returns artwork jobs for the NEW cards
/// at their post-sort positions. Runs on the Slint event loop.
pub fn append_release_page(
    window: &AppWindow,
    release_type: &str,
    new_cards: Vec<CardData>,
    server_has_more: bool,
) -> Vec<ArtworkJob> {
    let pages = LOADED_PAGES.with(|cell| {
        let mut m = cell.borrow_mut();
        let e = m.entry(release_type.to_string()).or_insert(1);
        *e += 1;
        *e
    });
    let mut jobs = Vec::new();
    let model = window.global::<ArtistState>().get_release_sections();
    for i in 0..model.row_count() {
        let Some(row) = model.row_data(i) else { continue };
        if row.release_type.as_str() != release_type {
            continue;
        }
        let sort = row.sort_by.to_string();
        let mut items: Vec<AlbumCardItem> = row.albums.iter().collect();
        let mut seen: HashSet<String> = items.iter().map(|a| a.id.to_string()).collect();
        let mut appended_ids: Vec<String> = Vec::new();
        for card in new_cards {
            let item = card_to_item(card);
            let id = item.id.to_string();
            if seen.contains(&id) {
                continue;
            }
            seen.insert(id.clone());
            appended_ids.push(id);
            items.push(item);
        }
        crate::album_map::sort_album_items(&mut items, &sort);
        let has_more = server_has_more && pages < MAX_INDEX_PAGES && !appended_ids.is_empty();
        for (idx, item) in items.iter().enumerate() {
            if appended_ids.iter().any(|id| id == item.id.as_str())
                && !item.artwork_url.as_str().is_empty()
            {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::ArtistRelease {
                        section_idx: i,
                        album_idx: idx,
                    },
                    url: item.artwork_url.to_string(),
                });
            }
        }
        let new_row = ArtistReleaseSection {
            albums: ModelRc::new(VecModel::from(items.clone())),
            has_more,
            ..row
        };
        model.set_row_data(i, new_row);
        FULL_RELEASE_SECTIONS.with(|cell| {
            for s in cell.borrow_mut().iter_mut() {
                if s.release_type.as_str() == release_type {
                    s.albums = ModelRc::new(VecModel::from(items.clone()));
                    s.has_more = has_more;
                    break;
                }
            }
        });
        break;
    }
    jobs
}
