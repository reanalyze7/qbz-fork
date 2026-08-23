use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoritesPreferences {
    pub custom_icon_path: Option<String>,
    pub custom_icon_preset: Option<String>,
    pub icon_background: Option<String>,
    pub tab_order: Vec<String>,
}

impl Default for FavoritesPreferences {
    fn default() -> Self {
        Self {
            custom_icon_path: None,
            custom_icon_preset: Some("heart".to_string()),
            icon_background: None,
            tab_order: vec![
                "tracks".to_string(),
                "albums".to_string(),
                "artists".to_string(),
                "playlists".to_string(),
            ],
        }
    }
}
