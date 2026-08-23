//! Follow-state reads/writes for the label header and the More-Labels
//! carousel cards.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, LabelState};

/// Current follow state for `label_id` — the header when it's the open
/// label, else the matching More-Labels card.
pub fn label_following_state(window: &AppWindow, label_id: &str) -> bool {
    let state = window.global::<LabelState>();
    if state.get_id().as_str() == label_id {
        return state.get_is_following();
    }
    let model = state.get_more_labels();
    for i in 0..model.row_count() {
        if let Some(item) = model.row_data(i) {
            if item.id.as_str() == label_id {
                return item.following;
            }
        }
    }
    false
}

/// Name of a More-Labels card by id (nav-history fallback for a card click).
pub fn more_label_name(window: &AppWindow, label_id: &str) -> String {
    let model = window.global::<LabelState>().get_more_labels();
    for i in 0..model.row_count() {
        if let Some(item) = model.row_data(i) {
            if item.id.as_str() == label_id {
                return item.title.to_string();
            }
        }
    }
    String::new()
}

/// Optimistically reflect a follow toggle — flips the header state when
/// it's the current label, and the matching more-labels card.
pub fn mark_label_followed(window: &AppWindow, label_id: &str, following: bool) {
    let state = window.global::<LabelState>();
    if state.get_id().as_str() == label_id {
        state.set_is_following(following);
    }
    let model = state.get_more_labels();
    for i in 0..model.row_count() {
        if let Some(mut item) = model.row_data(i) {
            if item.id.as_str() == label_id {
                item.following = following;
                model.set_row_data(i, item);
            }
        }
    }
}
