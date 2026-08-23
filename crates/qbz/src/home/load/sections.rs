//! Discover-index -> section-set assembly: blacklist filtering + the
//! Home / Editor's Picks `SectionData` lists + the "Most Streamed" slim grid.

use qbz_app::settings::discover_prefs::DiscoverySectionId;
use qbz_models::{DiscoverAlbum, DiscoverContainer, DiscoverContainers};

use crate::home::map::{map_slim, push_section, push_section_ref};
use crate::home::SectionData;

/// T8: drop blacklisted DiscoverAlbums (ANY of artists[], featured-aware)
/// from the discover-index containers. Tauri filters exactly these six
/// containers (ideal_discography, new_releases, qobuzissims, most_streamed,
/// press_awards, album_of_the_week) and adjusts NO count — log-only parity
/// (the carousels are has_more/cache-driven, not total-driven).
pub(super) fn apply_blacklist(containers: &mut DiscoverContainers) {
    let (bl, abl) = if crate::artist_blacklist::is_enabled() {
        (
            crate::artist_blacklist::ids_snapshot(),
            crate::artist_blacklist::album_ids_snapshot(),
        )
    } else {
        Default::default()
    };
    if bl.is_empty() && abl.is_empty() {
        return;
    }
    let retain = |c: &mut Option<DiscoverContainer<DiscoverAlbum>>| {
        if let Some(container) = c.as_mut() {
            container
                .data
                .items
                .retain(|a| !qbz_core::core::discover_album_blacklisted(a, &bl, &abl));
        }
    };
    retain(&mut containers.new_releases);
    retain(&mut containers.qobuzissims);
    retain(&mut containers.press_awards);
    retain(&mut containers.most_streamed);
    retain(&mut containers.ideal_discography);
    retain(&mut containers.album_of_the_week);
}

/// Editorial-only set for the Editor's Picks tab — built by cloning the
/// containers so the same data can also feed the Home set and the
/// most-streamed slim grid. Order mirrors Tauri's DEFAULT_PREFS.editorPicks.
pub(super) fn editor_sections(containers: &DiscoverContainers) -> Vec<SectionData> {
    let mut out = Vec::new();
    push_section_ref(&mut out, DiscoverySectionId::NewReleases, &qbz_i18n::t("New Releases"), "/discover/newReleases", &containers.new_releases);
    push_section_ref(&mut out, DiscoverySectionId::Qobuzissimes, &qbz_i18n::t("Qobuzissimes"), "/discover/qobuzissims", &containers.qobuzissims);
    push_section_ref(&mut out, DiscoverySectionId::PressAwards, &qbz_i18n::t("Press Accolades"), "/discover/pressAward", &containers.press_awards);
    push_section_ref(&mut out, DiscoverySectionId::MostStreamed, &qbz_i18n::t("Most Streamed"), "/discover/mostStreamed", &containers.most_streamed);
    push_section_ref(
        &mut out,
        DiscoverySectionId::IdealDiscography,
        &qbz_i18n::t("Ideal Discography"),
        "/discover/idealDiscography",
        &containers.ideal_discography,
    );
    push_section_ref(
        &mut out,
        DiscoverySectionId::EditorPicks,
        &qbz_i18n::t("Albums of the Week"),
        "/discover/albumOfTheWeek",
        &containers.album_of_the_week,
    );
    out
}

/// Home tab's section set — takes each field out of `containers` (leaving
/// the discover-playlist fields, read afterwards, untouched). Qobuzissimes
/// is kept in the cache pool even though it defaults OFF on Home, so
/// enabling it via the configurator has data to render.
pub(super) fn home_sections(containers: &mut DiscoverContainers) -> Vec<SectionData> {
    let mut out = Vec::new();
    push_section(&mut out, DiscoverySectionId::NewReleases, &qbz_i18n::t("New Releases"), "/discover/newReleases", containers.new_releases.take());
    push_section(&mut out, DiscoverySectionId::PressAwards, &qbz_i18n::t("Press Accolades"), "/discover/pressAward", containers.press_awards.take());
    push_section(
        &mut out,
        DiscoverySectionId::IdealDiscography,
        &qbz_i18n::t("Ideal Discography"),
        "/discover/idealDiscography",
        containers.ideal_discography.take(),
    );
    push_section(
        &mut out,
        DiscoverySectionId::EditorPicks,
        &qbz_i18n::t("Albums of the Week"),
        "/discover/albumOfTheWeek",
        containers.album_of_the_week.take(),
    );
    push_section(
        &mut out,
        DiscoverySectionId::Qobuzissimes,
        &qbz_i18n::t("Qobuzissimes"),
        "/discover/qobuzissims",
        containers.qobuzissims.take(),
    );
    out
}

/// "Most Streamed" mapped to the slim-grid rank list, capped at 24 (two
/// carousel pages of 12 — the slim carousel does not show beyond that).
pub(super) fn popular_slims(most_streamed: Option<DiscoverContainer<DiscoverAlbum>>) -> Vec<crate::home::SlimData> {
    most_streamed
        .map(|container| container.data.items)
        .unwrap_or_default()
        .into_iter()
        .take(24)
        .enumerate()
        .map(|(index, album)| map_slim(index, album))
        .collect()
}
