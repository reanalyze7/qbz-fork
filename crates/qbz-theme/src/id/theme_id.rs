use serde::{Deserialize, Serialize};

/// Every theme the registry can produce. The four marked "(P1)" are the only
/// ones materialized in Phase 1; the rest are placeholders the P2/P3 phases
/// fill in. `from_slug`/`slug` are stable across releases (persisted in
/// `ui_prefs.theme`), so the variant ORDER may change freely but the slugs must
/// not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeId {
    // --- Core ---
    Dark,   // P1 — :root
    Oled,   // P1 — DEFAULT theme
    Light,
    System, // P1 — meta (OS-following; resolved in the frontend)
    // Warm sepia/yellow paper tone — reduces blue light for eye comfort
    // (owner-requested "night light" style theme, à la e-reader sepia mode).
    Sepia,
    // --- Dark (branded / community) ---
    Warm,
    Nord,
    Dracula,
    TokyoNight, // P1
    CatppuccinMocha,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    BreezeDark,
    AdwaitaDark,
    Aurora,
    Ikari,
    Ayanami,
    Iscariot,
    Stratego,
    Rumi,
    Zoey,
    Mira,
    Frost,   // registered light, visually dark
    Langley, // registered light, visually dark
    // --- Light (branded / community) ---
    Alucard,
    RosePineDawn,
    BreezeLight,
    AdwaitaLight,
    DuotoneSnow,
    SnowStorm,
    Kurosaki,
    // --- Accessibility (REDESIGNED in P3) ---
    WcagLight,
    WcagDark,
    HighContrast,
    HighContrastLight,
    Colorblind,
}

/// All theme variants in display order (Core, Dark, Light, Accessibility).
pub const ALL: &[ThemeId] = &[
    ThemeId::Dark,
    ThemeId::Oled,
    ThemeId::Light,
    ThemeId::System,
    ThemeId::Sepia,
    ThemeId::Warm,
    ThemeId::Nord,
    ThemeId::Dracula,
    ThemeId::TokyoNight,
    ThemeId::CatppuccinMocha,
    ThemeId::CatppuccinLatte,
    ThemeId::CatppuccinFrappe,
    ThemeId::CatppuccinMacchiato,
    ThemeId::BreezeDark,
    ThemeId::AdwaitaDark,
    ThemeId::Aurora,
    ThemeId::Ikari,
    ThemeId::Ayanami,
    ThemeId::Iscariot,
    ThemeId::Stratego,
    ThemeId::Rumi,
    ThemeId::Zoey,
    ThemeId::Mira,
    ThemeId::Frost,
    ThemeId::Langley,
    ThemeId::Alucard,
    ThemeId::RosePineDawn,
    ThemeId::BreezeLight,
    ThemeId::AdwaitaLight,
    ThemeId::DuotoneSnow,
    ThemeId::SnowStorm,
    ThemeId::Kurosaki,
    ThemeId::WcagLight,
    ThemeId::WcagDark,
    ThemeId::HighContrast,
    ThemeId::HighContrastLight,
    ThemeId::Colorblind,
];
