//! Bindings (defaults + user overrides) + conflict detection, persisted
//! through `ui_prefs`.

use std::collections::BTreeMap;

use super::actions::{action, ActionDef, ACTIONS};

/// The active binding map (defaults overlaid with the user's overrides).
pub fn active_bindings() -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for a in ACTIONS {
        map.insert(a.id.to_string(), a.default.to_string());
    }
    let prefs = crate::ui_prefs::load();
    for (id, shortcut) in prefs.keybindings {
        if map.contains_key(&id) {
            map.insert(id, shortcut);
        }
    }
    map
}

pub(super) fn action_for_shortcut<'a>(
    shortcut: &str,
    bindings: &'a BTreeMap<String, String>,
) -> Option<&'static ActionDef> {
    let id = bindings
        .iter()
        .find(|(_, v)| v.as_str() == shortcut)
        .map(|(k, _)| k.clone())?;
    action(&id)
}

/// The action (other than `exclude`) that already owns `shortcut`, if any.
pub(super) fn conflicting_action(
    shortcut: &str,
    exclude: &str,
    bindings: &BTreeMap<String, String>,
) -> Option<&'static ActionDef> {
    for (id, sc) in bindings {
        if sc == shortcut && id != exclude {
            return action(id);
        }
    }
    None
}

/// Persist a new binding. Returns false (and writes nothing) on a conflict.
pub(super) fn set_binding(action_id: &str, shortcut: &str) -> bool {
    let bindings = active_bindings();
    if conflicting_action(shortcut, action_id, &bindings).is_some() {
        return false;
    }
    let default = action(action_id).map(|a| a.default);
    let mut prefs = crate::ui_prefs::load();
    if Some(shortcut) == default {
        // Back to default → drop the override (keeps the file minimal).
        prefs.keybindings.remove(action_id);
    } else {
        prefs.keybindings.insert(action_id.to_string(), shortcut.to_string());
    }
    crate::ui_prefs::save(&prefs);
    true
}

pub(super) fn reset_one(action_id: &str) {
    let mut prefs = crate::ui_prefs::load();
    prefs.keybindings.remove(action_id);
    crate::ui_prefs::save(&prefs);
}

pub(super) fn reset_all() {
    let mut prefs = crate::ui_prefs::load();
    prefs.keybindings.clear();
    crate::ui_prefs::save(&prefs);
}
