//! Ring-snapshot + filter + cap-to-[`MAX_VIEW_ROWS`] logic, and the plain-text
//! join used by `copy-all`.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, LogRow, LogViewerState};

use super::filter::{line_matches, MAX_VIEW_ROWS};

/// Snapshot + filter the ring, cap to the last [`MAX_VIEW_ROWS`], and push the
/// rows + counters onto `LogViewerState`. Runs on the UI thread.
pub(super) fn rebuild(weak: &slint::Weak<AppWindow>) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let st = w.global::<LogViewerState>();
    let level = st.get_filter_level().to_string().to_lowercase();
    let search = st.get_search().to_string().to_lowercase();

    let snap = qbz_log::ring::snapshot();
    let total = snap.len();
    let filtered: Vec<&qbz_log::LogLine> = snap
        .iter()
        .filter(|line| line_matches(line, &level, &search))
        .collect();
    let start = filtered.len().saturating_sub(MAX_VIEW_ROWS);
    let rows: Vec<LogRow> = filtered[start..]
        .iter()
        .map(|line| LogRow {
            ts: line.format_ts().into(),
            level: line.level_str().into(),
            target: line.target.clone().into(),
            message: line.message.clone().into(),
        })
        .collect();
    let shown = rows.len();

    st.set_rows(ModelRc::new(VecModel::from(rows)));
    st.set_total(total as i32);
    st.set_shown(shown as i32);
}

/// The currently-filtered rows as redacted `"{ts} {level} {target} {message}"`
/// lines (last [`MAX_VIEW_ROWS`]). Used by `copy-all`.
pub(super) fn filtered_text(weak: &slint::Weak<AppWindow>) -> Vec<String> {
    let Some(w) = weak.upgrade() else {
        return Vec::new();
    };
    let st = w.global::<LogViewerState>();
    let level = st.get_filter_level().to_string().to_lowercase();
    let search = st.get_search().to_string().to_lowercase();

    let snap = qbz_log::ring::snapshot();
    let filtered: Vec<&qbz_log::LogLine> = snap
        .iter()
        .filter(|line| line_matches(line, &level, &search))
        .collect();
    let start = filtered.len().saturating_sub(MAX_VIEW_ROWS);
    filtered[start..]
        .iter()
        .map(|line| {
            format!(
                "{} {} {} {}",
                line.format_ts(),
                line.level_str(),
                line.target,
                qbz_log::redact(&line.message)
            )
        })
        .collect()
}
