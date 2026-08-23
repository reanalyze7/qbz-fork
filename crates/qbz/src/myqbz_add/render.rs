//! Pure mapping from `LoadedRow` to `MyQbzAddRow`, plus the search-filter
//! rebuild.

use std::sync::{LazyLock, Mutex};

use qbz_models::mixtape::CollectionKind;
use slint::{ComponentHandle, ModelRc, VecModel};

use super::rows::LoadedRow;
use crate::{AppWindow, MyQbzAddRow, MyQbzAddState};

fn kind_icon(kind: CollectionKind) -> &'static str {
    match kind {
        CollectionKind::Mixtape => "cassette",
        CollectionKind::ArtistCollection => "user",
        CollectionKind::Collection => "library-big",
    }
}

fn kind_label(kind: CollectionKind) -> String {
    match kind {
        CollectionKind::Mixtape => qbz_i18n::t("MIXTAPE"),
        CollectionKind::Collection => qbz_i18n::t("COLLECTION"),
        CollectionKind::ArtistCollection => qbz_i18n::t("ARTIST"),
    }
}

/// "N albums" / "1 album" (always "album(s)" regardless of item_type — 1:1 PSD).
fn album_count_label(count: usize) -> String {
    qbz_i18n::tf("{} album", "{} albums", count as i64, &[&count.to_string()])
}

/// Last-loaded rows (so search filters client-side, no refetch).
static ROWS_CACHE: LazyLock<Mutex<Vec<LoadedRow>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Render loaded rows into `MyQbzAddState`, applying the active search filter.
/// UI thread.
pub fn apply_rows(window: &AppWindow, rows: Vec<LoadedRow>) {
    // Stash so a later search re-filters without a DB refetch.
    if let Ok(mut c) = ROWS_CACHE.lock() {
        *c = rows;
    }
    rebuild(window);
    window.global::<MyQbzAddState>().set_loading(false);
}

/// Rebuild the visible row model from the cache honoring the search filter.
pub fn rebuild(window: &AppWindow) {
    let state = window.global::<MyQbzAddState>();
    let query = state.get_search().trim().to_lowercase();
    let cache = ROWS_CACHE.lock();
    let items: Vec<MyQbzAddRow> = cache
        .as_ref()
        .map(|rows| {
            rows.iter()
                .filter(|r| query.is_empty() || r.name.to_lowercase().contains(&query))
                .map(|r| MyQbzAddRow {
                    id: r.id.clone().into(),
                    name: r.name.clone().into(),
                    kind: match r.kind {
                        CollectionKind::Mixtape => "mixtape",
                        CollectionKind::Collection => "collection",
                        CollectionKind::ArtistCollection => "artist_collection",
                    }
                    .into(),
                    icon: kind_icon(r.kind).into(),
                    kind_label: kind_label(r.kind).into(),
                    meta: album_count_label(r.item_count).into(),
                    already_has: r.already_has,
                })
                .collect()
        })
        .unwrap_or_default();
    state.set_rows(ModelRc::new(VecModel::from(items)));
}
