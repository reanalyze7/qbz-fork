use std::sync::{Arc, Mutex};

use qbz_app::playback_driver::DriverDeps;

use crate::state::DaemonShared;

/// Assemble the driver's host side channels: the streaming-quality resolver and
/// the daemon-shared latching / tick-timestamping hooks. T10: `on_edge` now
/// pulses the QConnect report `Notify` so the report scheduler reports on the
/// same transition/periodic edges the driver detects (§7.2).
pub(super) fn build_driver_deps(
    quality_cell: Arc<std::sync::Mutex<qbz_models::Quality>>,
    shared: Arc<Mutex<DaemonShared>>,
    report_notify: Arc<tokio::sync::Notify>,
) -> DriverDeps {
    let latch_shared = shared.clone();
    let tick_shared = shared;
    DriverDeps {
        quality: Arc::new(move || {
            quality_cell
                .lock()
                .map(|q| *q)
                .unwrap_or(qbz_models::Quality::UltraHiRes)
        }),
        // T10: signal the report scheduler on every ReportEdge. `notify_one`
        // stores a single permit if the scheduler is mid-report, so no edge is
        // lost and rapid edges coalesce into one report.
        on_edge: Arc::new(move || report_notify.notify_one()),
        on_latch: Arc::new(move |category, message| {
            if let Ok(mut s) = latch_shared.lock() {
                match category {
                    "stream" => s.last_errors.stream = Some(message),
                    "transport" => s.last_errors.transport = Some(message),
                    "auth" => s.last_errors.auth = Some(message),
                    _ => {}
                }
            }
        }),
        on_tick: Arc::new(move || {
            if let Ok(mut s) = tick_shared.lock() {
                s.driver_last_tick = Some(std::time::Instant::now());
            }
        }),
    }
}
