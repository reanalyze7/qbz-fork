use super::category::ThemeCategory;
use super::theme_id::ThemeId;

impl ThemeId {
    /// Category grouping for the Settings list. Note the corrected placement of
    /// `Frost`/`Langley` (visually dark despite the Tauri `type:light` flag) and
    /// `Alucard` (a genuine light theme grouped under "Dark" in Tauri).
    pub fn category(self) -> ThemeCategory {
        use ThemeId::*;
        match self {
            Dark | Oled | Light | System => ThemeCategory::Core,
            Warm | Nord | Dracula | TokyoNight | CatppuccinMocha | CatppuccinFrappe
            | CatppuccinMacchiato | BreezeDark | AdwaitaDark
            | Aurora | Ikari | Ayanami | Iscariot | Stratego | Rumi | Zoey | Mira | Frost
            | Langley => ThemeCategory::Dark,
            // Catppuccin Latte is the LIGHT flavor — groups with the light themes.
            Alucard | CatppuccinLatte | RosePineDawn | BreezeLight | AdwaitaLight | DuotoneSnow
            | SnowStorm | Kurosaki | Sepia => ThemeCategory::Light,
            WcagLight | WcagDark | HighContrast | HighContrastLight | Colorblind => {
                ThemeCategory::Accessibility
            }
        }
    }
}
