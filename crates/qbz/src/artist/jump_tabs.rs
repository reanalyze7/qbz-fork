use crate::JumpNavTab;

/// Build the JUMP TO tab list for this artist. Tabs are emitted only
/// for sections that actually have content (no empty Compilations
/// row when the artist has none); each tab carries a page-local
/// `anchor-y` estimate so a click can scroll the page-flickable
/// straight to that section. Heights are layout-derived
/// approximations — variable bio length and grid wrapping make a
/// truly precise number hard without measuring each frame, but the
/// estimates land the user inside the right section.
pub(crate) fn build_jump_tabs(
    top_tracks_count: usize,
    has_last_release: bool,
    sections: &[(String, usize)],
    appears_on_count: usize,
) -> Vec<JumpNavTab> {
    // Layout constants — keep in sync with ArtistPageView.slint.
    const BODY_ROW_TOP_GUESS: f32 = 320.0;
    const SECTION_SPACER: f32 = 32.0;
    const RELEASE_HEADER: f32 = 28.0;
    const RELEASE_ROW: f32 = 290.0;
    const RELEASE_ROW_GAP: f32 = 24.0;
    const RELEASE_COLS: f32 = 5.0;
    const POPULAR_HEADER: f32 = 36.0;
    const POPULAR_HEADER_GAP: f32 = 10.0;
    const POPULAR_ROW: f32 = 52.0;
    const POPULAR_TAIL: f32 = 32.0;
    // "Novedad más reciente" highlight block (header + one card row).
    const LAST_RELEASE_BLOCK: f32 = 172.0;

    let mut tabs: Vec<JumpNavTab> = Vec::new();
    tabs.push(JumpNavTab {
        id: "about".into(),
        label: qbz_i18n::t("About").into(),
        anchor_y: 0.0,
    });

    let mut cursor = BODY_ROW_TOP_GUESS;
    if top_tracks_count > 0 {
        tabs.push(JumpNavTab {
            id: "popular-tracks".into(),
            label: qbz_i18n::t("Popular Tracks").into(),
            anchor_y: cursor,
        });
        let visible_rows = top_tracks_count.min(5) as f32;
        cursor +=
            POPULAR_HEADER + POPULAR_HEADER_GAP + visible_rows * POPULAR_ROW + POPULAR_TAIL;
    }

    // The latest-release highlight has no jump tab (it's a highlight, not a
    // browsable section) but it shifts every section below it.
    if has_last_release {
        cursor += LAST_RELEASE_BLOCK;
    }

    for (title, count) in sections {
        // Route by display title → stable jump-tab id. Unknown titles still
        // render as sections; they just don't get a jump tab.
        let id = match title.as_str() {
            "Albums" => "albums",
            "EPs & Singles" => "eps-singles",
            "Live" => "live",
            "Compilations" => "compilations",
            "Purchase Only" => "purchase-only",
            "Composer" => "composer",
            // "Other" is rendered LAST + collapsed (below Appears On), so it
            // gets no jump tab and does not occupy main-flow height here.
            "Critics' Picks" => "critics-picks",
            "Upcoming" => "upcoming",
            _ => continue,
        };
        tabs.push(JumpNavTab {
            id: id.into(),
            // `title` is the English bucket title used for id routing above;
            // translate only the displayed label.
            label: qbz_i18n::t(title).into(),
            anchor_y: cursor,
        });
        let rows = (*count as f32 / RELEASE_COLS).ceil().max(1.0);
        cursor += SECTION_SPACER
            + RELEASE_HEADER
            + rows * RELEASE_ROW
            + (rows - 1.0).max(0.0) * RELEASE_ROW_GAP;
    }

    if appears_on_count > 0 {
        tabs.push(JumpNavTab {
            id: "appears-on".into(),
            label: qbz_i18n::t("Appears On").into(),
            anchor_y: cursor,
        });
    }

    tabs
}
