//! Process-wide clipboard singleton.

/// Long-lived clipboard instance. arboard ties the offer's lifetime to the
/// LAST live `Clipboard` object: dropping it destroys the X11 selection
/// window (contents survive only when a clipboard MANAGER accepts the
/// handoff — KDE ships one, stock GNOME/XFCE/Cinnamon do not) and ends the
/// Wayland offer with the same rule. The old create-per-copy pattern
/// therefore worked on KDE and silently lost the text everywhere else
/// (HiFi-wizard copy report, #514). One instance kept alive for the whole
/// process serves the offer like any normal app.
static CLIPBOARD: std::sync::OnceLock<std::sync::Mutex<Option<arboard::Clipboard>>> =
    std::sync::OnceLock::new();

/// Copy `text` to the system clipboard. Runs on a blocking thread —
/// clipboard backends (X11/Wayland) can block.
pub fn copy_to_clipboard(text: String) {
    tokio::task::spawn_blocking(move || {
        let cell = CLIPBOARD.get_or_init(|| std::sync::Mutex::new(None));
        let mut guard = match cell.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if guard.is_none() {
            match arboard::Clipboard::new() {
                Ok(c) => *guard = Some(c),
                Err(e) => {
                    log::warn!("[qbz-slint] clipboard unavailable: {e}");
                    return;
                }
            }
        }
        if let Some(clipboard) = guard.as_mut() {
            if let Err(e) = clipboard.set_text(text) {
                log::warn!("[qbz-slint] clipboard set failed: {e}");
                // Drop the instance so the next copy reconnects — the
                // display connection may have gone away.
                *guard = None;
            }
        }
    });
}
