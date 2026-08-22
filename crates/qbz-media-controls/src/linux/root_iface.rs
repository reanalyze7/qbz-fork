use std::sync::{Arc, Mutex};

use mpris_server::zbus::{self, fdo};
use mpris_server::RootInterface;

use crate::types::MediaEvent;

use super::{EventCb, State, DESKTOP_ENTRY, IDENTITY};

/// The MPRIS interface implementation. Getters read shared `State`; the action
/// methods forward to the app via `on_event`.
pub(super) struct QbzMpris {
    pub(super) on_event: EventCb,
    pub(super) state: Arc<Mutex<State>>,
}

impl QbzMpris {
    pub(super) fn emit(&self, ev: MediaEvent) {
        (self.on_event)(ev);
    }
}

impl RootInterface for QbzMpris {
    async fn raise(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Raise);
        Ok(())
    }
    async fn quit(&self) -> fdo::Result<()> {
        self.emit(MediaEvent::Quit);
        Ok(())
    }
    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn set_fullscreen(&self, _fullscreen: bool) -> zbus::Result<()> {
        Ok(())
    }
    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(true)
    }
    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn identity(&self) -> fdo::Result<String> {
        Ok(IDENTITY.to_string())
    }
    /// The GNOME app-icon fix: GNOME resolves the icon from this property.
    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok(DESKTOP_ENTRY.to_string())
    }
    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}
