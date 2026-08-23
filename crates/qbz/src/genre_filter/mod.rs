//! Filter-by-genre controller.
//!
//! Loads the parent genres for the popup's simple grid and owns the genre
//! selection. The selection is **per context** ("discover" for the three
//! Discover tabs, "favorites" for the favorites tabs) so the two surfaces
//! filter independently (Tauri keeps them separate too). The popup edits
//! whatever context is `current` (set when it opens). The selection
//! persists to `<data-dir>/qbz/genre_filter.json` when "Remember
//! selection" is on, and feeds `genre_ids` into the discover-index fetch /
//! the favorites client-side genre filter.

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

mod apply;
mod context;
mod loaders;
mod mutations;
mod persistence;
mod tree;

pub use apply::apply_state;
pub use context::{current_context, selected_ids, selected_ids_for, selected_names, set_context};
pub use loaders::{children_loaded, load_all_parent_children, load_children, load_descendants, load_parents};
pub use mutations::{clear, set_remember, toggle};
pub use tree::{set_search, toggle_expand};

#[derive(Clone)]
pub(self) struct GenreItem {
    id: u64,
    name: String,
}

pub(self) struct State {
    parents: Vec<GenreItem>,
    /// Lazily loaded children, keyed by parent id (levels 2 and 3).
    children: HashMap<u64, Vec<GenreItem>>,
    /// Selected genre ids per context.
    selected: HashMap<String, Vec<u64>>,
    /// The context the popup is currently editing.
    current: String,
    expanded: HashSet<u64>,
    search: String,
    remember: bool,
}

impl State {
    /// Mutable handle to the current context's selection (created if absent).
    fn cur_mut(&mut self) -> &mut Vec<u64> {
        let key = self.current.clone();
        self.selected.entry(key).or_default()
    }
    fn is_selected(&self, id: u64) -> bool {
        self.selected
            .get(&self.current)
            .map(|v| v.contains(&id))
            .unwrap_or(false)
    }
    fn cur_len(&self) -> usize {
        self.selected
            .get(&self.current)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

pub(self) static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| {
    Mutex::new(State {
        parents: Vec::new(),
        children: HashMap::new(),
        selected: HashMap::new(),
        current: "discover".to_string(),
        expanded: HashSet::new(),
        search: String::new(),
        remember: true,
    })
});
