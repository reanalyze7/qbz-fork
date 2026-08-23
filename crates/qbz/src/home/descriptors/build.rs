//! Slice-5 prefs-driven descriptor-list builders: cached `SectionData` +
//! `DiscoverPrefs` -> the `SectionDescriptor` lists the Home/Editor's Picks
//! repeater renders, plus their artwork jobs.

use qbz_app::settings::discover_prefs::{DiscoverPrefs, DiscoverySectionId, DiscoveryTab};
use slint::{ModelRc, SharedString, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{DiscoverSection, SectionDescriptor};

use super::super::present::card_to_item;
use super::super::SectionData;
use super::renderable::HOME_RENDERABLE;

/// Build one Slint `DiscoverSection` from cached album data (mirrors
/// `present::build_sections` for a single entry).
fn descriptor_section(data: &SectionData) -> DiscoverSection {
    DiscoverSection {
        title: data.title.clone().into(),
        endpoint: data.endpoint.clone().into(),
        albums: ModelRc::new(VecModel::from(
            data.albums.iter().cloned().map(card_to_item).collect::<Vec<_>>(),
        )),
    }
}

/// Build one tab's ordered ENABLED descriptor list from `prefs` + the cached
/// section data. Album-carousel ids embed their `DiscoverSection` (Home/Editor
/// share the Carousel component but have no per-id HomeState field). The
/// fixed-data ids (qobuzPlaylists / continueListening / mostStreamed-slim) and
/// the always-present-with-placeholder ids (recentlyPlayedAlbums on Home) bind
/// HomeState fields in the view; they carry an empty `section` — as do the
/// #566 ported rails: favoriteAlbums / releaseWatch / topArtists bind their
/// HomeState fields and self-hide while empty; qobuzMixes is static
/// navigation tiles, always rendered when enabled (Tauri parity).
///
/// **Empty-section policy (b):** an album-carousel id with no cached data is
/// DROPPED (no backing `SectionData` → nothing to render, and these have no
/// placeholder). recentlyPlayedAlbums / continueListening / qobuzPlaylists /
/// mostStreamed are KEPT (the view arm self-hides or shows a placeholder on
/// empty data), preserving the 1:1 Home placeholders. This keeps every mounted
/// album-carousel delegate non-empty (the documented anti-spacing-doubling form).
pub(super) fn descriptors_for(prefs: &DiscoverPrefs, tab: DiscoveryTab, cached: &[SectionData]) -> Vec<SectionDescriptor> {
    use DiscoverySectionId::*;
    let editor = tab == DiscoveryTab::EditorPicks;
    let mut out = Vec::new();
    for id in prefs.enabled_ordered(tab) {
        // #566 structural guard: skip ids the HomeView repeater has no arm
        // for (stale persisted prefs) instead of emitting an invisible row.
        if !HOME_RENDERABLE.contains(&id) {
            continue;
        }
        // mostStreamed renders as an album carousel on Editor's Picks, a slim
        // grid on Home — encode that in `kind` so the delegate dispatches it
        // without reading active-tab.
        let kind = if id == MostStreamed {
            if editor { "albumCarousel" } else { "slimGrid" }
        } else {
            crate::discover_prefs::render_kind(id)
        };
        // Album-carousel ids that pull from the cached SectionData.
        let is_album_cache = matches!(
            id,
            NewReleases | PressAwards | IdealDiscography | EditorPicks | Qobuzissimes
        ) || (id == MostStreamed && editor);
        let section = if is_album_cache {
            match cached.iter().find(|s| s.id == id) {
                Some(data) => descriptor_section(data),
                // Empty-section policy (b): no data for this album id → drop it.
                None => continue,
            }
        } else {
            DiscoverSection::default()
        };
        out.push(SectionDescriptor {
            id: SharedString::from(id.as_str()),
            kind: SharedString::from(kind),
            section,
        });
    }
    out
}

/// Artwork jobs for a tab's descriptor list — they target the embedded album
/// sections in `DiscoverState.home-sections` / `editor-sections` (NOT
/// HomeState.sections), so covers paint on the prefs-driven Home/Editor loop.
/// Built from the cached `SectionData` (CardData urls) keyed by the descriptor
/// id, so `section_idx` aligns with the descriptor's position in the pushed
/// list — no need to read back the Slint model.
pub(super) fn discover_section_artwork_jobs(
    descriptors: &[SectionDescriptor],
    cached: &[SectionData],
    editor: bool,
) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    for (section_idx, desc) in descriptors.iter().enumerate() {
        // Only album-carousel descriptors map to a cached SectionData; the
        // fixed-data ids have no entry and contribute no jobs here.
        let Some(data) = cached.iter().find(|s| s.id.as_str() == desc.id.as_str()) else {
            continue;
        };
        for (album_idx, card) in data.albums.iter().enumerate() {
            if card.artwork_url.is_empty() {
                continue;
            }
            jobs.push(ArtworkJob {
                target: ArtworkTarget::DiscoverSectionAlbum {
                    editor,
                    section_idx,
                    album_idx,
                },
                url: card.artwork_url.clone(),
            });
        }
    }
    jobs
}
