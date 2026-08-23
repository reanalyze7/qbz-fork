//! Slint model (cheatsheet + customize editor share the groups).

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, KeybindingsState};

use super::actions::{Category, Context, ACTIONS};
use super::bindings::active_bindings;
use super::grammar::format_display;

pub(super) fn build_group_vec() -> Vec<crate::KeybindingCategoryGroup> {
    let bindings = active_bindings();
    let mut groups: Vec<crate::KeybindingCategoryGroup> = Vec::new();
    for cat in Category::ORDER {
        let mut rows: Vec<crate::KeybindingRow> = Vec::new();
        for a in ACTIONS.iter().filter(|a| a.category == cat) {
            let shortcut = bindings.get(a.id).cloned().unwrap_or_default();
            let modified = bindings.get(a.id).map(|s| s.as_str()) != Some(a.default);
            rows.push(crate::KeybindingRow {
                id: a.id.into(),
                label: qbz_i18n::t(a.label_en).into(),
                shortcut: format_display(&shortcut).into(),
                modified,
                contextual: a.context != Context::None,
            });
        }
        groups.push(crate::KeybindingCategoryGroup {
            label: qbz_i18n::t(cat.label_en()).into(),
            rows: ModelRc::new(VecModel::from(rows)),
        });
    }
    groups
}

pub(super) fn modified_count() -> i32 {
    let bindings = active_bindings();
    ACTIONS
        .iter()
        .filter(|a| bindings.get(a.id).map(|s| s.as_str()) != Some(a.default))
        .count() as i32
}

/// Repopulate the Slint state from the persisted bindings. Call at startup and
/// after any change.
pub fn refresh(window: &AppWindow) {
    let state = window.global::<KeybindingsState>();
    let groups = build_group_vec();
    // Full list (customize editor renders one column) + a round-robin split into
    // three columns for the read-only cheatsheet (avoids one tall scroll).
    let mut cols: [Vec<crate::KeybindingCategoryGroup>; 3] = Default::default();
    for (i, g) in groups.iter().enumerate() {
        cols[i % 3].push(g.clone());
    }
    let [c0, c1, c2] = cols;
    state.set_groups(ModelRc::new(VecModel::from(groups)));
    state.set_groups_col1(ModelRc::new(VecModel::from(c0)));
    state.set_groups_col2(ModelRc::new(VecModel::from(c1)));
    state.set_groups_col3(ModelRc::new(VecModel::from(c2)));
    state.set_modified_count(modified_count());
}
