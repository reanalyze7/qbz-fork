//! `DiagController::export_clipboard` — serialize the cached snapshot (+
//! `exportedAt`) to the clipboard, flipping `copied` for 1.5s.

use serde_json::Value;
use slint::ComponentHandle;

use crate::DiagnosticsState;

use super::super::DiagController;

impl DiagController {
    pub(in crate::diagnostics) fn export_clipboard(&self) {
        let base = self.export.lock().ok().and_then(|g| g.clone());
        let Some(mut value) = base else {
            return;
        };
        if let Some(map) = value.as_object_mut() {
            map.insert(
                "exportedAt".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }
        let json = serde_json::to_string_pretty(&value).unwrap_or_default();
        crate::share::copy_to_clipboard(json);

        if let Some(w) = self.weak.upgrade() {
            w.global::<DiagnosticsState>().set_copied(true);
        }
        let weak = self.weak.clone();
        self.handle.spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            let _ = weak.upgrade_in_event_loop(|w| {
                w.global::<DiagnosticsState>().set_copied(false);
            });
        });
    }
}
