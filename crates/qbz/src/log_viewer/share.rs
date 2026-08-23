//! Clipboard flash + the shareable diagnostics-bundle text builder, used by
//! the copy-all / copy-bundle / upload / copy-url callback bodies.

use slint::ComponentHandle;

use crate::{AppWindow, LogViewerState};

use super::auto_tail::AUTO_TAIL_INTERVAL;
use super::Runtime;

/// Wire the `copy-bundle` / `upload` / `open-log-file` / `copy-url`
/// callbacks — the remaining `LogViewerState` handlers after `install()`
/// wires the refresh/filter/auto-tail/copy-all group itself.
pub(super) fn wire_bundle_callbacks(
    window: &AppWindow,
    runtime: Runtime,
    handle: tokio::runtime::Handle,
) {
    let state = window.global::<LogViewerState>();
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        let runtime = runtime.clone();
        state.on_copy_bundle(move || {
            let weak = weak.clone();
            let runtime = runtime.clone();
            handle.spawn(async move {
                let bundle = build_share_text(&runtime).await;
                crate::share::copy_to_clipboard(bundle);
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<LogViewerState>().set_copied(true);
                });
                tokio::time::sleep(AUTO_TAIL_INTERVAL).await;
                let _ = weak.upgrade_in_event_loop(|w| {
                    w.global::<LogViewerState>().set_copied(false);
                });
            });
        });
    }
    {
        let weak = window.as_weak();
        state.on_upload(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<LogViewerState>().set_uploading(true);
            }
            let weak = weak.clone();
            let runtime = runtime.clone();
            handle.spawn(async move {
                let bundle = build_share_text(&runtime).await;
                let url = match reqwest::Client::new()
                    .post("https://paste.rs/")
                    .body(bundle)
                    .send()
                    .await
                {
                    Ok(resp) => resp.text().await.unwrap_or_default().trim().to_string(),
                    Err(e) => {
                        log::warn!("[qbz-slint] log upload failed: {e}");
                        String::new()
                    }
                };
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<LogViewerState>();
                    st.set_uploaded_url(url.into());
                    st.set_uploading(false);
                });
            });
        });
    }
    {
        state.on_open_log_file(move || {
            if let Some(path) = qbz_log::install::log_file_path() {
                if let Err(e) = open::that(path) {
                    log::warn!("[qbz-slint] open log file failed: {e}");
                }
            }
        });
    }
    {
        let weak = window.as_weak();
        state.on_copy_url(move || {
            if let Some(w) = weak.upgrade() {
                let url = w.global::<LogViewerState>().get_uploaded_url().to_string();
                if !url.is_empty() {
                    crate::share::copy_to_clipboard(url);
                }
            }
        });
    }
}

/// Flash `copied = true` for the standard window, then reset on a tokio timer.
pub(super) fn flash_copied(weak: &slint::Weak<AppWindow>, handle: &tokio::runtime::Handle) {
    if let Some(w) = weak.upgrade() {
        w.global::<LogViewerState>().set_copied(true);
    }
    let weak = weak.clone();
    handle.spawn(async move {
        tokio::time::sleep(AUTO_TAIL_INTERVAL).await;
        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<LogViewerState>().set_copied(false);
        });
    });
}

/// Build the COMPLETE shareable diagnostics text used by both "Copy diagnostics
/// bundle" and "Upload (public)": the full diagnostics report (system + the LIVE
/// active audio device + graphics + playback + qconnect) followed by the last 200
/// redacted log lines. This is what makes the uploaded paste complete rather than
/// "just logs". All log lines are already redacted at the ring's write choke
/// point; `qbz_log::redact` is applied again defensively.
pub(super) async fn build_share_text(runtime: &Runtime) -> String {
    let report = crate::diagnostics::build_full_report(runtime).await;

    let lines = qbz_log::ring::snapshot();
    let start = lines.len().saturating_sub(200);
    let mut logs = String::new();
    for line in &lines[start..] {
        logs.push_str(&format!(
            "{} {} {} {}\n",
            line.format_ts(),
            line.level_str(),
            line.target,
            qbz_log::redact(&line.message)
        ));
    }

    format!("{report}\n\n## Recent logs\n\n```log\n{logs}```\n")
}
