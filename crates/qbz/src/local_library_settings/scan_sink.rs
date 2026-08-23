//! The `ScanEvent` sink: coalesces the per-file event stream (~100ms) and
//! pushes throttled progress to `LibraryScanState`.

use std::sync::Mutex;

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibraryScanState};

fn basename(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .to_string()
}

/// ~100ms coalescing for the per-file event stream (a 16K-track scan would
/// otherwise flood the event loop). Terminal/phase events bypass this.
fn throttle_ok(last: &Mutex<std::time::Instant>) -> bool {
    let mut g = last.lock().unwrap_or_else(|e| e.into_inner());
    if g.elapsed() >= std::time::Duration::from_millis(100) {
        *g = std::time::Instant::now();
        true
    } else {
        false
    }
}

/// Build the progress-event sink closure for one scan run.
pub(super) fn make_sink(
    weak_sink: Weak<AppWindow>,
    last: Mutex<std::time::Instant>,
) -> impl Fn(qbz_library::ScanEvent) {
    move |ev: qbz_library::ScanEvent| {
        use qbz_library::ScanEvent::*;
        match ev {
            Started => {}
            TotalsAdded { total } => {
                let _ = weak_sink.upgrade_in_event_loop(move |w| {
                    w.global::<LibraryScanState>().set_total_files(total as i32);
                });
            }
            FileStarted { path } => {
                if throttle_ok(&last) {
                    let base = basename(&path);
                    let _ = weak_sink.upgrade_in_event_loop(move |w| {
                        w.global::<LibraryScanState>().set_current_file(base.into());
                    });
                }
            }
            FileDone { processed, total } => {
                if throttle_ok(&last) {
                    let _ = weak_sink.upgrade_in_event_loop(move |w| {
                        let s = w.global::<LibraryScanState>();
                        s.set_processed_files(processed as i32);
                        s.set_total_files(total as i32);
                        s.set_progress(if total > 0 {
                            (processed as f32 / total as f32).min(1.0)
                        } else {
                            0.0
                        });
                    });
                }
            }
            Cleanup => {
                let _ = weak_sink.upgrade_in_event_loop(|w| {
                    w.global::<LibraryScanState>()
                        .set_current_file(qbz_i18n::t("Cleaning up missing files...").into());
                });
            }
            Finished { status, errors } => {
                let st = match status {
                    qbz_library::ScanStatus::Complete => 2,
                    qbz_library::ScanStatus::Cancelled => 3,
                    qbz_library::ScanStatus::Error => 4,
                    _ => 0,
                };
                let ec = errors.len() as i32;
                let _ = weak_sink.upgrade_in_event_loop(move |w| {
                    let s = w.global::<LibraryScanState>();
                    s.set_scanning(false);
                    s.set_scan_status(st);
                    s.set_error_count(ec);
                    s.set_current_file("".into());
                    if st == 2 {
                        s.set_progress(1.0);
                    }
                });
                match st {
                    2 if ec > 0 => crate::toast::success_weak(
                        &weak_sink,
                        qbz_i18n::tf(
                            "Scan complete ({} file skipped)",
                            "Scan complete ({} files skipped)",
                            ec as i64,
                            &[&ec.to_string()],
                        ),
                    ),
                    2 => crate::toast::success_weak(&weak_sink, qbz_i18n::t("Scan complete")),
                    3 => crate::toast::success_weak(&weak_sink, qbz_i18n::t("Scan cancelled")),
                    _ => crate::toast::error_weak(&weak_sink, qbz_i18n::t("Scan failed")),
                }
            }
        }
    }
}
