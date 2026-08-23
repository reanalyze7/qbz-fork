use super::context::PlaybackContext;
use std::sync::Mutex;

pub struct ContextManager {
    current: Mutex<Option<PlaybackContext>>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(None),
        }
    }

    pub fn set_context(&self, context: PlaybackContext) {
        let mut current = self.current.lock().unwrap();
        *current = Some(context);
        log::info!(
            "Playback context set: {:?}",
            current.as_ref().map(|c| &c.label)
        );
    }

    pub fn clear_context(&self) {
        let mut current = self.current.lock().unwrap();
        *current = None;
        log::info!("Playback context cleared");
    }

    pub fn get_context(&self) -> Option<PlaybackContext> {
        self.current.lock().unwrap().clone()
    }

    pub fn has_context(&self) -> bool {
        self.current.lock().unwrap().is_some()
    }

    pub fn next_track_id(&self) -> Option<u64> {
        self.current
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|ctx| ctx.next_track_id())
    }

    pub fn upcoming_track_ids(&self, count: usize) -> Vec<u64> {
        self.current
            .lock()
            .unwrap()
            .as_ref()
            .map(|ctx| ctx.upcoming_track_ids(count))
            .unwrap_or_default()
    }

    pub fn advance_context(&self) -> bool {
        let mut current = self.current.lock().unwrap();
        if let Some(ctx) = current.as_mut() {
            let advanced = ctx.advance();
            if !advanced {
                log::info!("Playback context ended (no more tracks)");
            }
            advanced
        } else {
            false
        }
    }

    pub fn set_position(&self, track_id: u64) {
        let mut current = self.current.lock().unwrap();
        if let Some(ctx) = current.as_mut() {
            if let Some(pos) = ctx.track_ids.iter().position(|&id| id == track_id) {
                ctx.current_position = pos;
                log::debug!("Context position updated to {}", pos);
            }
        }
    }

    pub fn append_track_ids(&self, new_track_ids: Vec<u64>) {
        let mut current = self.current.lock().unwrap();
        if let Some(ctx) = current.as_mut() {
            let count = new_track_ids.len();
            ctx.track_ids.extend(new_track_ids);
            log::debug!(
                "Appended {} track IDs to context (total: {})",
                count,
                ctx.track_ids.len()
            );
        }
    }
}
