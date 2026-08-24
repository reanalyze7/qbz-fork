use crate::*;
use crate::navigate_album_artist::nav_statics::LAST_CORTINILLA;

pub(crate) fn wire_search_part4(window: &AppWindow, _app_runtime: &Arc<AppRuntime<SlintAdapter>>, _tokio_rt: &tokio::runtime::Runtime, _image_cache: &artwork::ImageCache, _settings_ctx: &Arc<settings::SettingsCtx>) {

    // Cortinilla: arrow-key move the keyboard highlight (delta -1 up / +1 down).
    // The valid navigable flat indices are NOT guaranteed to be a contiguous
    // 0..=max range (when there is no top result, index 0 is skipped and the
    // section rows start at 1), so the order is rebuilt from the live snapshot:
    // the top-result's flat index first (when present), then every section row's
    // flat index in declaration order. `selected-index == -1` means "nothing
    // highlighted" (Enter falls through to search-all); Down from -1 lands on the
    // first row, Up from the first row returns to -1. Both ends clamp (no wrap).
    {
        let weak = window.as_weak();
        window
            .global::<SearchActions>()
            .on_cortinilla_move_selection(move |delta| {
                let Some(w) = weak.upgrade() else { return };
                // Build the ordered list of navigable flat indices.
                let order: Vec<i32> = LAST_CORTINILLA.with(|c| {
                    let snap = c.borrow();
                    let Some(data) = snap.as_ref() else {
                        return Vec::new();
                    };
                    let mut v: Vec<i32> = Vec::new();
                    if let Some(top) = &data.top {
                        v.push(top.flat_index as i32);
                    }
                    for section in &data.sections {
                        for row in &section.rows {
                            v.push(row.flat_index as i32);
                        }
                    }
                    v
                });
                if order.is_empty() {
                    return;
                }
                let st = w.global::<SearchState>();
                let current = st.get_selected_index();
                // Current position within the navigable order (-1 if nothing /
                // stale value not present anymore).
                let pos = order.iter().position(|&fi| fi == current);
                let new_index: i32 = if delta > 0 {
                    // Down: from "nothing" -> first; otherwise advance, clamping
                    // at the last row.
                    match pos {
                        None => order[0],
                        Some(p) if p + 1 < order.len() => order[p + 1],
                        Some(_) => order[order.len() - 1],
                    }
                } else {
                    // Up: from "nothing" stay nothing; from the first row -> -1;
                    // otherwise step back.
                    match pos {
                        None => -1,
                        Some(0) => -1,
                        Some(p) => order[p - 1],
                    }
                };
                st.set_selected_index(new_index);
                // Content-top y of the selected row so the overlay can scroll it
                // into view. Mirrors Cortinilla.slint's layout EXACTLY: top-result
                // block = padTop(4) + label(22) + row(56); each section block =
                // padTop(4) + header(24) + rows × 56. 0 when nothing is selected.
                let scroll_y: f32 = if new_index < 0 {
                    0.0
                } else {
                    LAST_CORTINILLA.with(|c| {
                        let snap = c.borrow();
                        let Some(data) = snap.as_ref() else {
                            return 0.0;
                        };
                        let mut y: f32 = 0.0;
                        if let Some(top) = &data.top {
                            if top.flat_index as i32 == new_index {
                                return y + 26.0; // padTop 4 + label 22
                            }
                            y += 82.0; // 4 + 22 + 56
                        }
                        for section in &data.sections {
                            y += 28.0; // padTop 4 + header 24
                            for row in &section.rows {
                                if row.flat_index as i32 == new_index {
                                    return y;
                                }
                                y += 56.0;
                            }
                        }
                        0.0
                    })
                };
                st.set_cortinilla_scroll_y(scroll_y);
            });
    }
}
