//! Push the current popup state into the `GenreFilterState` Slint global —
//! the one function in this module that touches `AppWindow`.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, GenreChip, GenreFilterState};

use super::tree::build_tree_rows;
use super::STATE;

/// Push the current parents + selection + tree into GenreFilterState (for
/// the current context). UI thread.
pub fn apply_state(window: &AppWindow) {
    let (chips, rows, count, remember) = {
        let Ok(s) = STATE.lock() else {
            return;
        };
        let chips: Vec<GenreChip> = s
            .parents
            .iter()
            .map(|g| GenreChip {
                id: g.id.to_string().into(),
                name: g.name.clone().into(),
                selected: s.is_selected(g.id),
            })
            .collect();
        (chips, build_tree_rows(&s), s.cur_len() as i32, s.remember)
    };
    let state = window.global::<GenreFilterState>();
    state.set_genres(ModelRc::new(VecModel::from(chips)));
    state.set_tree(ModelRc::new(VecModel::from(rows)));
    state.set_selected_count(count);
    state.set_remember(remember);
}
