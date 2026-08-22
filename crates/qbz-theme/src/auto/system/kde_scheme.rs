//! KDE full color-scheme reading (kdeglobals), split out of `kde.rs` for the
//! 130-line budget. Shares `read_kde_color_key` with the accent-color probe.

use std::env;
use std::fs;

use super::super::SystemColorScheme;
use super::kde::read_kde_color_key;

pub(super) fn get_kde_color_scheme() -> Result<SystemColorScheme, String> {
    let home = env::var("HOME").map_err(|_| "HOME not set".to_string())?;
    let config_path = format!("{}/.config/kdeglobals", home);
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("Cannot read kdeglobals: {}", e))?;

    let accent_explicit = read_kde_color_key(&content, "[General]", "AccentColor");

    let scheme = SystemColorScheme {
        window_bg: read_kde_color_key(&content, "[Colors:Window]", "BackgroundNormal"),
        window_bg_alt: read_kde_color_key(&content, "[Colors:Window]", "BackgroundAlternate"),
        view_bg: read_kde_color_key(&content, "[Colors:View]", "BackgroundNormal"),
        button_bg: read_kde_color_key(&content, "[Colors:Button]", "BackgroundNormal"),
        header_bg: read_kde_color_key(&content, "[Colors:Header]", "BackgroundNormal"),
        header_bg_inactive: read_kde_color_key(
            &content,
            "[Colors:Header][Inactive]",
            "BackgroundNormal",
        ),
        tooltip_bg: read_kde_color_key(&content, "[Colors:Tooltip]", "BackgroundNormal"),

        window_fg: read_kde_color_key(&content, "[Colors:Window]", "ForegroundNormal"),
        window_fg_inactive: read_kde_color_key(&content, "[Colors:Window]", "ForegroundInactive"),
        view_fg: read_kde_color_key(&content, "[Colors:View]", "ForegroundNormal"),
        button_fg: read_kde_color_key(&content, "[Colors:Button]", "ForegroundNormal"),

        selection_bg: read_kde_color_key(&content, "[Colors:Selection]", "BackgroundNormal"),
        selection_fg: read_kde_color_key(&content, "[Colors:Selection]", "ForegroundNormal"),
        selection_hover: read_kde_color_key(&content, "[Colors:Selection]", "DecorationHover"),
        accent: accent_explicit
            .or_else(|| read_kde_color_key(&content, "[Colors:Selection]", "DecorationFocus")),

        fg_link: read_kde_color_key(&content, "[Colors:Window]", "ForegroundLink"),
        fg_negative: read_kde_color_key(&content, "[Colors:Window]", "ForegroundNegative"),
        fg_neutral: read_kde_color_key(&content, "[Colors:Window]", "ForegroundNeutral"),
        fg_positive: read_kde_color_key(&content, "[Colors:Window]", "ForegroundPositive"),

        wm_active_bg: read_kde_color_key(&content, "[WM]", "activeBackground"),
        wm_active_fg: read_kde_color_key(&content, "[WM]", "activeForeground"),
        wm_inactive_bg: read_kde_color_key(&content, "[WM]", "inactiveBackground"),
    };

    if scheme.window_bg.is_none() {
        return Err("KDE color scheme missing Colors:Window BackgroundNormal".into());
    }

    Ok(scheme)
}
