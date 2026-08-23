use super::store::GraphicsSettingsStore;
use rusqlite::params;

impl GraphicsSettingsStore {
    pub fn set_hardware_acceleration(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET hardware_acceleration = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set hardware_acceleration: {}", e))?;
        Ok(())
    }

    pub fn set_force_x11(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET force_x11 = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set force_x11: {}", e))?;
        Ok(())
    }

    pub fn set_gdk_scale(&self, value: Option<String>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET gdk_scale = ?1 WHERE id = 1",
                params![value],
            )
            .map_err(|e| format!("Failed to set gdk_scale: {}", e))?;
        Ok(())
    }

    pub fn set_gdk_dpi_scale(&self, value: Option<String>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET gdk_dpi_scale = ?1 WHERE id = 1",
                params![value],
            )
            .map_err(|e| format!("Failed to set gdk_dpi_scale: {}", e))?;
        Ok(())
    }

    pub fn set_gsk_renderer(&self, value: Option<String>) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET gsk_renderer = ?1 WHERE id = 1",
                params![value],
            )
            .map_err(|e| format!("Failed to set gsk_renderer: {}", e))?;
        Ok(())
    }

    pub fn set_preferred_gpu(&self, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET preferred_gpu = ?1 WHERE id = 1",
                params![value],
            )
            .map_err(|e| format!("Failed to set preferred_gpu: {}", e))?;
        Ok(())
    }

    pub fn set_nvidia_compat_mode(&self, enabled: bool) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE graphics_settings SET nvidia_compat_mode = ?1 WHERE id = 1",
                params![enabled as i64],
            )
            .map_err(|e| format!("Failed to set nvidia_compat_mode: {}", e))?;
        Ok(())
    }
}
