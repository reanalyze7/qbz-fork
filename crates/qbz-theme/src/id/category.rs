/// Grouping shown in the Settings theme list. Mirrors the Tauri SettingsView
/// registry comment blocks (Core / Dark / Light / Accessibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeCategory {
    Core,
    Dark,
    Light,
    Accessibility,
}

impl ThemeCategory {
    pub fn slug(self) -> &'static str {
        match self {
            ThemeCategory::Core => "core",
            ThemeCategory::Dark => "dark",
            ThemeCategory::Light => "light",
            ThemeCategory::Accessibility => "accessibility",
        }
    }
}
