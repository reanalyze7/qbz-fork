use super::theme_id::{ThemeId, ALL};

impl ThemeId {
    /// Stable persisted slug. MUST NOT change once shipped.
    pub fn slug(self) -> &'static str {
        match self {
            ThemeId::Dark => "dark",
            ThemeId::Oled => "oled",
            ThemeId::Light => "light",
            ThemeId::System => "system",
            ThemeId::Sepia => "sepia",
            ThemeId::Warm => "warm",
            ThemeId::Nord => "nord",
            ThemeId::Dracula => "dracula",
            ThemeId::TokyoNight => "tokyo-night",
            ThemeId::CatppuccinMocha => "catppuccin-mocha",
            ThemeId::CatppuccinLatte => "catppuccin-latte",
            ThemeId::CatppuccinFrappe => "catppuccin-frappe",
            ThemeId::CatppuccinMacchiato => "catppuccin-macchiato",
            ThemeId::BreezeDark => "breeze-dark",
            ThemeId::AdwaitaDark => "adwaita-dark",
            ThemeId::Aurora => "aurora",
            ThemeId::Ikari => "ikari",
            ThemeId::Ayanami => "ayanami",
            ThemeId::Iscariot => "iscariot",
            ThemeId::Stratego => "stratego",
            ThemeId::Rumi => "rumi",
            ThemeId::Zoey => "zoey",
            ThemeId::Mira => "mira",
            ThemeId::Frost => "frost",
            ThemeId::Langley => "langley",
            ThemeId::Alucard => "alucard",
            ThemeId::RosePineDawn => "rose-pine-dawn",
            ThemeId::BreezeLight => "breeze-light",
            ThemeId::AdwaitaLight => "adwaita-light",
            ThemeId::DuotoneSnow => "duotone-snow",
            ThemeId::SnowStorm => "snow-storm",
            ThemeId::Kurosaki => "kurosaki",
            ThemeId::WcagLight => "wcag-light",
            ThemeId::WcagDark => "wcag-dark",
            ThemeId::HighContrast => "high-contrast",
            ThemeId::HighContrastLight => "high-contrast-light",
            ThemeId::Colorblind => "colorblind",
        }
    }

    /// Parse a persisted slug back to a `ThemeId`. Unknown slugs return `None`
    /// (the caller falls back to the default).
    pub fn from_slug(s: &str) -> Option<ThemeId> {
        ALL.iter().copied().find(|id| id.slug() == s)
    }
}
