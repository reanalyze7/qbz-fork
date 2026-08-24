use crate::*;

/// Open a MusicianPageView for `name + role`. Routes to the artist
/// page instead when the resolved musician has a Confirmed Qobuz
/// match (Tauri's `confidence === 'confirmed'` shortcut). Fetches
/// the first page of appearances inline; subsequent pages come
/// through the MusicianActions::load-more handler.
pub(crate) fn navigate_musician(
    runtime: Arc<AppRuntime<SlintAdapter>>,
    weak: slint::Weak<AppWindow>,
    handle: &tokio::runtime::Handle,
    image_cache: artwork::ImageCache,
    name: String,
    role: String,
) {
    handle.spawn(async move {
        let _ = weak.upgrade_in_event_loop(|w| {
            musician::reset_musician(&w);
            w.global::<NavState>().set_view(ContentView::Musician);
        });
        match musician::load_musician(&runtime, &name, &role).await {
            Ok(data) => {
                let jobs = musician::artwork_jobs(&data);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    musician::apply_musician(&w, data);
                });
                artwork::spawn_loads(jobs, weak.clone(), image_cache.clone());
            }
            Err(e) => {
                log::error!("[qbz-slint] musician load failed: {e}");
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<MusicianState>().set_loading(false);
                });
            }
        }
    });
}

/// Resolve the desktop environment's UI font family. Reads the KDE
/// Plasma general font from `kdeglobals` (`[General] font=`), whose
/// value is a Qt font string `Family,pointSize,...` — only the family
/// is taken. Returns `None` off KDE or when the key is absent, so the
/// caller can fall back to the Slint default.
pub(crate) fn system_font_family() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let path = std::path::Path::new(&home).join(".config/kdeglobals");
    let content = std::fs::read_to_string(path).ok()?;
    let mut in_general = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_general = line == "[General]";
            continue;
        }
        if in_general {
            if let Some(rest) = line.strip_prefix("font=") {
                let family = rest.split(',').next().unwrap_or("").trim();
                if !family.is_empty() {
                    return Some(family.to_string());
                }
            }
        }
    }
    None
}

