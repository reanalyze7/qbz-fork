use super::theme_id::ThemeId;

impl ThemeId {
    /// The default theme on a fresh profile (owner decision 2026-06-20).
    pub const fn default_id() -> ThemeId {
        ThemeId::Oled
    }

    /// Whether this theme is fully materialized by the registry. Used by the
    /// frontend list builder to expose only ready themes during the phased
    /// rollout. After P3 every theme — including the 5 redesigned accessibility
    /// themes — is materialized, so this is now unconditionally `true`.
    pub fn is_implemented(self) -> bool {
        true
    }
}

/// The default theme slug (`"oled"`). Convenience for `ui_prefs::default_theme`.
pub fn default_slug() -> &'static str {
    ThemeId::default_id().slug()
}
