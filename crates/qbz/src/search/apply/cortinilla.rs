use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::search::rows::{CortRow, CortinillaData};
use crate::{
    AppWindow, CortinillaRow as CortinillaRowItem, CortinillaSection as CortinillaSectionItem,
    SearchState,
};

/// Turn a plain `CortRow` into its Slint item. `artwork` starts empty; the
/// artwork pipeline resolves it in place keyed off `flat_index`.
fn cortinilla_row_item(row: &CortRow) -> CortinillaRowItem {
    CortinillaRowItem {
        kind: row.kind.clone().into(),
        id: row.id.clone().into(),
        source: row.source.clone().into(),
        title: row.title.clone().into(),
        subtitle: row.subtitle.clone().into(),
        artwork_url: row.artwork_url.clone().into(),
        artwork: slint::Image::default(),
        flat_index: row.flat_index as i32,
    }
}

/// Write a cortinilla payload into `SearchState`. Runs on the Slint event loop.
/// Clears `cortinilla-loading`. Does NOT reset `selected-index` here — the live
/// handler resets selection only when the query actually changed, so a late
/// revalidation overwrite keeps the user's current highlight.
pub fn apply_cortinilla(window: &AppWindow, data: CortinillaData) {
    let state = window.global::<SearchState>();
    state.set_cortinilla_query(data.query.clone().into());

    // Top result — an empty CortinillaRow (kind == "") means "no top result".
    match &data.top {
        Some(top) => state.set_top_result(cortinilla_row_item(top)),
        // An all-default row (kind == "", id == "") is the overlay's "no top
        // result" sentinel.
        None => state.set_top_result(CortinillaRowItem::default()),
    }

    let sections: Vec<CortinillaSectionItem> = data
        .sections
        .iter()
        .map(|s| {
            let rows: Vec<CortinillaRowItem> = s.rows.iter().map(cortinilla_row_item).collect();
            CortinillaSectionItem {
                title: s.title.clone().into(),
                kind: s.kind.clone().into(),
                rows: ModelRc::new(VecModel::from(rows)),
                has_more: s.has_more,
            }
        })
        .collect();
    state.set_sections(ModelRc::new(VecModel::from(sections)));
    state.set_cortinilla_loading(false);
}

/// Build artwork download jobs for a cortinilla payload — one per row that
/// carries a URL, keyed by the row's stable `flat_index` (top-result = 0).
pub fn cortinilla_artwork_jobs(data: &CortinillaData) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    if let Some(top) = &data.top {
        jobs.extend(crate::search::artwork::simple_job(
            ArtworkTarget::CortinillaRow {
                flat_index: top.flat_index,
            },
            &top.artwork_url,
        ));
    }
    for section in &data.sections {
        for row in &section.rows {
            jobs.extend(crate::search::artwork::simple_job(
                ArtworkTarget::CortinillaRow {
                    flat_index: row.flat_index,
                },
                &row.artwork_url,
            ));
        }
    }
    jobs
}

/// Write an immersive-search payload into `ImmersiveState`. Runs on the Slint
/// event loop. Builds the `[CortinillaSection]` model from `data.sections`
/// (reusing the shared row/section item-builders) and clears `search-loading`.
/// Does NOT touch `search-selected-index` — the controller owns selection and
/// resets it on every open/refine, so a late revalidation must not clobber it.
/// The immersive payload has no top result, so none is written.
pub fn apply_immersive_search(window: &AppWindow, data: &CortinillaData) {
    let state = window.global::<crate::ImmersiveState>();
    let sections: Vec<CortinillaSectionItem> = data
        .sections
        .iter()
        .map(|s| {
            let rows: Vec<CortinillaRowItem> = s.rows.iter().map(cortinilla_row_item).collect();
            CortinillaSectionItem {
                title: s.title.clone().into(),
                kind: s.kind.clone().into(),
                rows: ModelRc::new(VecModel::from(rows)),
                has_more: s.has_more,
            }
        })
        .collect();
    state.set_search_sections(ModelRc::new(VecModel::from(sections)));
    state.set_search_loading(false);
}

/// Build artwork download jobs for an immersive-search payload — one per row
/// that carries a URL, keyed by the row's stable `flat_index`. Targets
/// `ImmersiveSearchRow` (the immersive global) instead of `CortinillaRow`. The
/// immersive payload has no top result, so only section rows produce jobs.
pub fn immersive_cortinilla_artwork_jobs(data: &CortinillaData) -> Vec<ArtworkJob> {
    let mut jobs = Vec::new();
    for section in &data.sections {
        for row in &section.rows {
            jobs.extend(crate::search::artwork::simple_job(
                ArtworkTarget::ImmersiveSearchRow {
                    flat_index: row.flat_index,
                },
                &row.artwork_url,
            ));
        }
    }
    jobs
}
