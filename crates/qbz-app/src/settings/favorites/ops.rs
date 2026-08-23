use super::prefs::FavoritesPreferences;
use super::store::FavoritesPreferencesStore;
use rusqlite::params;

impl FavoritesPreferencesStore {
    pub fn get_preferences(&self) -> Result<FavoritesPreferences, String> {
        let mut stmt = self.conn.prepare("SELECT custom_icon_path, custom_icon_preset, icon_background, tab_order FROM favorites_preferences WHERE id = 1")
            .map_err(|e| format!("Failed to prepare select: {}", e))?;

        let result = stmt.query_row([], |row| {
            let custom_icon_path: Option<String> = row.get(0)?;
            let custom_icon_preset: Option<String> = row.get(1)?;
            let icon_background: Option<String> = row.get(2)?;
            let tab_order_str: String = row.get(3)?;

            let custom_icon_path = custom_icon_path.and_then(|value| {
                if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            });

            let tab_order: Vec<String> =
                serde_json::from_str(&tab_order_str).unwrap_or_else(|_| {
                    vec![
                        "tracks".to_string(),
                        "albums".to_string(),
                        "artists".to_string(),
                        "playlists".to_string(),
                    ]
                });

            Ok(FavoritesPreferences {
                custom_icon_path,
                custom_icon_preset,
                icon_background,
                tab_order,
            })
        });

        match result {
            Ok(mut prefs) => {
                if let Some(path) = prefs.custom_icon_path.clone() {
                    match self.normalize_custom_icon_path(path) {
                        Ok(resolved) => {
                            let normalized = if resolved.trim().is_empty() {
                                None
                            } else {
                                Some(resolved)
                            };
                            if normalized != prefs.custom_icon_path {
                                prefs.custom_icon_path = normalized;
                                let _ = self.save_preferences(prefs.clone());
                            }
                        }
                        Err(_) => {
                            prefs.custom_icon_path = None;
                            let _ = self.save_preferences(prefs.clone());
                        }
                    }
                }
                Ok(prefs)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(FavoritesPreferences::default()),
            Err(e) => Err(format!("Failed to query preferences: {}", e)),
        }
    }

    pub fn save_preferences(
        &self,
        mut prefs: FavoritesPreferences,
    ) -> Result<FavoritesPreferences, String> {
        if let Some(path) = prefs.custom_icon_path.clone() {
            let resolved = self.normalize_custom_icon_path(path)?;
            if resolved.is_empty() {
                prefs.custom_icon_path = None;
            } else {
                prefs.custom_icon_path = Some(resolved);
            }
        }

        let tab_order_str = serde_json::to_string(&prefs.tab_order)
            .map_err(|e| format!("Failed to serialize tab_order: {}", e))?;

        self.conn.execute(
            "INSERT OR REPLACE INTO favorites_preferences (id, custom_icon_path, custom_icon_preset, icon_background, tab_order)
             VALUES (1, ?1, ?2, ?3, ?4)",
            params![prefs.custom_icon_path, prefs.custom_icon_preset, prefs.icon_background, tab_order_str],
        )
        .map_err(|e| format!("Failed to save preferences: {}", e))?;
        Ok(prefs)
    }
}
