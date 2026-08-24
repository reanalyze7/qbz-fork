use crate::*;

/// Open the folder editor modal for an existing folder, populating
/// `FolderEditState` from the stored record. Shared by the Playlist
/// Manager edit-folder action and the sidebar context menu so both open
/// the same editor. The icon-preset/color grids are populated once at
/// startup, so the editor works from anywhere.
pub(crate) fn open_folder_editor(window: &AppWindow, id: slint::SharedString) {
    let fid = id.to_string();
    if let Some(f) = playlist_manager::folder_for_edit(&fid) {
        let fes = window.global::<FolderEditState>();
        fes.set_id(id);
        fes.set_name(f.name.into());
        fes.set_icon_preset(f.icon_preset.into());
        fes.set_icon_color(f.icon_color.into());
        fes.set_is_hidden(f.is_hidden);
        fes.set_custom_image_path(f.custom_image_path.clone().unwrap_or_default().into());
        fes.set_open(true);
        // Decode the existing custom image, if any.
        if let Some(path) = f.custom_image_path {
            playlist_manager::load_editor_custom_image(window.as_weak(), path);
        }
    }
}

/// Re-fire the artwork pipeline for the Playlist Manager's currently
/// rendered cards (after a rebuild swaps the models).
pub(crate) fn refresh_pm_covers(window: &AppWindow) {
    if let Some(cache) = artwork::shared_cache() {
        let jobs = playlist_manager::artwork_jobs(window);
        if !jobs.is_empty() {
            artwork::spawn_loads(jobs, window.as_weak(), cache);
        }
        let handle = tokio::runtime::Handle::current();
        playlist_manager::load_folder_custom_images(window.as_weak(), &handle);
    }
}

/// Build the folder-editor icon-preset + solid-color models (matches
/// Tauri's FolderEditModal presets). Run once when wiring the editor.
pub(crate) fn folder_editor_presets() -> (Vec<PmIconPreset>, Vec<PmColorSwatch>) {
    // The icon glyphs are resolved in the .slint by id (a `@image-url`
    // chain keyed on `preset.id`), so the model only carries the id; the
    // image field stays default.
    let presets: Vec<PmIconPreset> =
        ["heart", "star", "music", "folder", "disc", "library", "headphones"]
            .iter()
            .map(|id| PmIconPreset {
                id: (*id).into(),
                icon: slint::Image::default(),
            })
            .collect();

    let parse = |hex: &str| -> slint::Color {
        let h = hex.trim_start_matches('#');
        let v = u32::from_str_radix(h, 16).unwrap_or(0);
        slint::Color::from_rgb_u8(
            ((v >> 16) & 0xff) as u8,
            ((v >> 8) & 0xff) as u8,
            (v & 0xff) as u8,
        )
    };
    let mut swatches = vec![PmColorSwatch {
        value: "".into(),
        color: slint::Color::default(),
        is_accent: true,
    }];
    for hex in [
        "#ef4444", "#f97316", "#f59e0b", "#10b981", "#06b6d4", "#3b82f6", "#a855f7", "#ec4899",
        "#f43f5e", "#64748b",
    ] {
        swatches.push(PmColorSwatch {
            value: hex.into(),
            color: parse(hex),
            is_accent: false,
        });
    }
    (presets, swatches)
}

