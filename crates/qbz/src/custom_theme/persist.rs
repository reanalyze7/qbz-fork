//! `custom_theme.json` load/save + first-selection seeding.

use std::path::PathBuf;

use qbz_theme::CustomThemeBase;

/// Resolve `<data_dir>/qbz/custom_theme.json` (same dir as `ui_prefs.json`).
fn custom_theme_path() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("qbz").join("custom_theme.json"))
}

/// Load the persisted custom base. A missing/unreadable/corrupt file degrades to
/// the OLED-derived default rather than erroring (matches `ui_prefs::load`).
pub fn load() -> CustomThemeBase {
    let Some(path) = custom_theme_path() else {
        return CustomThemeBase::default_oled();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("[qbz-slint] custom_theme.json parse failed, using default: {e}");
            CustomThemeBase::default_oled()
        }),
        Err(_) => CustomThemeBase::default_oled(),
    }
}

/// Persist the custom base. Best-effort — failures are logged (matches
/// `ui_prefs::save`).
pub fn save(base: &CustomThemeBase) {
    let Some(path) = custom_theme_path() else {
        log::warn!("[qbz-slint] custom_theme.json: data dir unavailable, not saving");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::error!("[qbz-slint] custom_theme.json: create dir failed: {e}");
            return;
        }
    }
    match serde_json::to_string_pretty(base) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                log::error!("[qbz-slint] custom_theme.json: write failed: {e}");
            }
        }
        Err(e) => log::error!("[qbz-slint] custom_theme.json: serialize failed: {e}"),
    }
}

/// True when a persisted custom base exists on disk (first-selection check).
pub fn exists() -> bool {
    custom_theme_path().map(|p| p.exists()).unwrap_or(false)
}

/// Load the persisted base, or seed + persist the OLED default when no file
/// exists yet (the first time the user selects the "Custom" theme).
pub fn load_or_seed() -> CustomThemeBase {
    let exists = custom_theme_path().map(|p| p.exists()).unwrap_or(false);
    if exists {
        load()
    } else {
        let base = CustomThemeBase::default_oled();
        save(&base);
        base
    }
}
