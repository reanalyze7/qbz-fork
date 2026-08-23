//! Building the JUMP TO tabs for the landing page.

use super::LabelPagePayload;
use crate::JumpNavTab;

/// Build the JUMP TO tabs for the landing — only sections with content.
/// anchor-y values are layout-derived estimates (variable header/grid
/// heights make exact numbers impractical; the estimate lands the user
/// inside the right section). Mirrors artist::build_jump_tabs.
pub(super) fn build_label_jump_tabs(payload: &LabelPagePayload) -> Vec<JumpNavTab> {
    const HEADER_GUESS: f32 = 360.0;
    const SECTION_SPACER: f32 = 40.0;
    const CAROUSEL: f32 = 320.0;
    const POPULAR_HEADER: f32 = 46.0;
    const POPULAR_ROW: f32 = 50.0;
    const POPULAR_TAIL: f32 = 40.0;

    let mut tabs: Vec<JumpNavTab> = Vec::new();
    tabs.push(JumpNavTab {
        id: "about".into(),
        label: qbz_i18n::t("About").into(),
        anchor_y: 0.0,
    });
    let mut cursor = HEADER_GUESS;

    if !payload.top_tracks.is_empty() {
        tabs.push(JumpNavTab {
            id: "popular".into(),
            label: qbz_i18n::t("Popular Tracks").into(),
            anchor_y: cursor,
        });
        let rows = payload.top_tracks.len().min(5) as f32;
        cursor += POPULAR_HEADER + rows * POPULAR_ROW + POPULAR_TAIL;
    }
    let push_carousel = |tabs: &mut Vec<JumpNavTab>, id: &str, label: &str, present: bool, cursor: &mut f32| {
        if present {
            tabs.push(JumpNavTab {
                id: id.into(),
                label: qbz_i18n::t(label).into(),
                anchor_y: *cursor,
            });
            *cursor += SECTION_SPACER + CAROUSEL;
        }
    };
    // Labels are `mark`ed so the extractor registers the English literals; the
    // closure translates them once with `t(label)`.
    push_carousel(&mut tabs, "releases", qbz_i18n::mark("Releases"), !payload.releases.is_empty(), &mut cursor);
    push_carousel(&mut tabs, "critics", qbz_i18n::mark("Critics' Picks"), !payload.critics.is_empty(), &mut cursor);
    push_carousel(&mut tabs, "playlists", qbz_i18n::mark("Playlists"), !payload.playlists.is_empty(), &mut cursor);
    push_carousel(&mut tabs, "artists", qbz_i18n::mark("Artists"), !payload.artists.is_empty(), &mut cursor);
    push_carousel(&mut tabs, "labels", qbz_i18n::mark("More Labels"), !payload.more_labels.is_empty(), &mut cursor);
    tabs
}
