//! Async network-backed genre-tree loaders.

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;

use super::persistence::load_persisted;
use super::{GenreItem, STATE};

pub fn children_loaded(id: u64) -> bool {
    STATE.lock().map(|s| s.children.contains_key(&id)).unwrap_or(false)
}

fn store_children(parent_id: u64, kids: Vec<GenreItem>) {
    if let Ok(mut s) = STATE.lock() {
        s.children.insert(parent_id, kids);
    }
}

/// Fetch the parent genres (if not already loaded) and seed the persisted
/// selection. Runs on a worker; call apply_state afterwards on the UI
/// thread.
pub async fn load_parents<A>(runtime: &AppRuntime<A>)
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    {
        let already = STATE.lock().map(|s| !s.parents.is_empty()).unwrap_or(false);
        if already {
            return;
        }
    }
    let persisted = load_persisted();
    let mut parents: Vec<GenreItem> = match runtime.core().get_genres(None).await {
        Ok(list) => list
            .into_iter()
            .map(|g| GenreItem {
                id: g.id,
                name: g.name,
            })
            .collect(),
        Err(e) => {
            log::warn!("[qbz-slint] genre filter: get_genres failed: {e}");
            Vec::new()
        }
    };
    parents.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Keep persisted selections as-is — they may reference child genres
    // not yet loaded (advanced view), so validating against parents only
    // would wrongly drop them.
    if let Ok(mut s) = STATE.lock() {
        s.parents = parents;
        let mut contexts = persisted.contexts;
        // Migrate a legacy flat selection into the discover context.
        if contexts.is_empty() && !persisted.selected.is_empty() {
            contexts.insert("discover".to_string(), persisted.selected);
        }
        s.selected = contexts;
        s.remember = persisted.remember;
    }
}

/// Load one genre level (children of `parent_id`) and store it. No-op if
/// already loaded.
pub async fn load_children<A>(runtime: &AppRuntime<A>, parent_id: u64)
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    if children_loaded(parent_id) {
        return;
    }
    let kids: Vec<GenreItem> = match runtime.core().get_genres(Some(parent_id)).await {
        Ok(list) => list
            .into_iter()
            .map(|g| GenreItem {
                id: g.id,
                name: g.name,
            })
            .collect(),
        Err(e) => {
            log::warn!("[qbz-slint] genre filter: get_genres({parent_id}) failed: {e}");
            Vec::new()
        }
    };
    store_children(parent_id, kids);
}

fn child_ids(parent_id: u64) -> Vec<u64> {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.children.get(&parent_id).map(|k| k.iter().map(|c| c.id).collect()))
        .unwrap_or_default()
}

/// Eager-load every parent's children (level 2) so the advanced tree can
/// show child counts up front. Grandchildren stay lazy.
pub async fn load_all_parent_children<A>(runtime: &AppRuntime<A>)
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let parents: Vec<u64> = STATE
        .lock()
        .map(|s| s.parents.iter().map(|p| p.id).collect())
        .unwrap_or_default();
    for parent_id in parents {
        load_children(runtime, parent_id).await;
    }
}

/// Eager-load a genre's full descendant subtree (children + grandchildren),
/// so a selection expands correctly in selected_names (favorites) and the
/// tree shows counts.
pub async fn load_descendants<A>(runtime: &AppRuntime<A>, id: u64)
where
    A: FrontendAdapter + Send + Sync + 'static,
{
    load_children(runtime, id).await;
    for kid in child_ids(id) {
        load_children(runtime, kid).await;
    }
}
