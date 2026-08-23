use super::store::PlaybackPreferencesStore;
use super::types::{AutoplayMode, PlaybackPreferences};
use rusqlite::params;

impl PlaybackPreferencesStore {
    pub fn set_autoplay_mode(&self, mode: AutoplayMode) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE playback_preferences SET autoplay_mode = ?1 WHERE id = 1",
                params![mode.to_db_value()],
            )
            .map_err(|e| format!("Failed to set autoplay mode: {}", e))?;
        Ok(())
    }

    pub fn set_show_context_icon(&self, show: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE playback_preferences SET show_context_icon = ?1 WHERE id = 1",
                params![if show { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set show context icon: {}", e))?;
        Ok(())
    }

    pub fn set_persist_session(&self, persist: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE playback_preferences SET persist_session = ?1 WHERE id = 1",
                params![if persist { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set persist session: {}", e))?;
        Ok(())
    }

    pub fn set_resume_playback_position(&self, resume: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE playback_preferences SET resume_playback_position = ?1 WHERE id = 1",
                params![if resume { 1 } else { 0 }],
            )
            .map_err(|e| format!("Failed to set resume playback position: {}", e))?;
        Ok(())
    }

    /// Reset all playback preferences to their default values.
    pub fn reset_all(&self) -> Result<PlaybackPreferences, String> {
        let defaults = PlaybackPreferences::default();
        self.conn
            .execute(
                "UPDATE playback_preferences SET autoplay_mode = ?1, show_context_icon = ?2, persist_session = ?3, resume_playback_position = ?4 WHERE id = 1",
                params![
                    defaults.autoplay_mode.to_db_value(),
                    if defaults.show_context_icon { 1 } else { 0 },
                    if defaults.persist_session { 1 } else { 0 },
                    if defaults.resume_playback_position { 1 } else { 0 },
                ],
            )
            .map_err(|e| format!("Failed to reset playback preferences: {}", e))?;
        Ok(defaults)
    }
}
