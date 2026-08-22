use super::theme_id::ThemeId;

impl ThemeId {
    /// Human-facing display name. This is proper-noun DATA (theme names like
    /// "Nord", "Tokyo Night", "OLED Black") — NOT a translatable UI string, so
    /// it lives here in the registry, not in the i18n catalog.
    pub fn display_name(self) -> &'static str {
        match self {
            ThemeId::Dark => "Dark",
            ThemeId::Oled => "OLED Black",
            ThemeId::Light => "Light",
            ThemeId::System => "System",
            ThemeId::Sepia => "Sepia (Eye Comfort)",
            ThemeId::Warm => "Warm",
            ThemeId::Nord => "Nord",
            ThemeId::Dracula => "Dracula",
            ThemeId::TokyoNight => "Tokyo Night",
            ThemeId::CatppuccinMocha => "Catppuccin Mocha",
            ThemeId::CatppuccinLatte => "Catppuccin Latte",
            ThemeId::CatppuccinFrappe => "Catppuccin Frappé",
            ThemeId::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeId::BreezeDark => "Breeze Dark",
            ThemeId::AdwaitaDark => "Adwaita Dark",
            ThemeId::Aurora => "Aurora",
            ThemeId::Ikari => "Ikari",
            ThemeId::Ayanami => "Ayanami",
            ThemeId::Iscariot => "Iscariot",
            ThemeId::Stratego => "Stratego",
            ThemeId::Rumi => "Rumi",
            ThemeId::Zoey => "Zoey",
            ThemeId::Mira => "Mira",
            ThemeId::Frost => "Frost",
            ThemeId::Langley => "Langley",
            ThemeId::Alucard => "Alucard",
            ThemeId::RosePineDawn => "Rose Pine Dawn",
            ThemeId::BreezeLight => "Breeze Light",
            ThemeId::AdwaitaLight => "Adwaita Light",
            ThemeId::DuotoneSnow => "Duotone Snow",
            ThemeId::SnowStorm => "Snow Storm",
            ThemeId::Kurosaki => "Kurosaki",
            ThemeId::WcagLight => "WCAG Light",
            ThemeId::WcagDark => "WCAG Dark",
            ThemeId::HighContrast => "High Contrast",
            ThemeId::HighContrastLight => "High Contrast Light",
            ThemeId::Colorblind => "Colorblind",
        }
    }
}
