use super::model::PlaybackState;

impl PlaybackState {
    /// Changed dotted keys for write_one. NEVER emits `ask` (§3.3.2).
    pub fn save_keys(&self) -> Vec<(String, String)> {
        let b = &self.baseline;
        let a = &self.staged;
        let mut out = Vec::new();
        if a.quality != b.quality {
            out.push(("playback.quality".to_string(), a.quality.clone()));
        }
        if a.limit_to_device != b.limit_to_device {
            out.push(("audio.limit_quality_to_device".to_string(), a.limit_to_device.to_string()));
        }
        if a.max_sample_rate != b.max_sample_rate {
            out.push((
                "audio.device_max_sample_rate".to_string(),
                a.max_sample_rate.map(|r| r.to_string()).unwrap_or_else(|| "none".to_string()),
            ));
        }
        if a.allow_fallback != b.allow_fallback {
            out.push(("audio.allow_quality_fallback".to_string(), a.allow_fallback.to_string()));
        }
        // Only write fallback_behavior when it is a concrete value (never `ask`).
        if a.fallback_behavior != b.fallback_behavior && a.fallback_behavior != "ask" {
            out.push((
                "audio.quality_fallback_behavior".to_string(),
                a.fallback_behavior.clone(),
            ));
        }
        if a.autoplay != b.autoplay {
            out.push(("playback.autoplay".to_string(), a.autoplay.clone()));
        }
        if a.gapless != b.gapless {
            out.push(("audio.gapless_enabled".to_string(), a.gapless.to_string()));
        }
        if a.restore_session != b.restore_session {
            out.push(("playback.persist_session".to_string(), a.restore_session.to_string()));
        }
        if a.resume_position != b.resume_position {
            out.push(("playback.resume_playback_position".to_string(), a.resume_position.to_string()));
        }
        if a.mpris != b.mpris {
            out.push(("playback.mpris".to_string(), a.mpris.to_string()));
        }
        out
    }
}
