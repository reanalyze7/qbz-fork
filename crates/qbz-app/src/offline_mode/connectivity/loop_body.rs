use std::time::{Duration, Instant, SystemTime};
use tokio::sync::watch;

use super::judge::ConnectivityJudge;
use super::probe::probe_all;
use super::route::has_default_route;
use super::types::{ConnectivitySnapshot, JudgeAction};

/// Seconds of audio-segment silence within which we are online by definition.
const LIVENESS_WINDOW_SECS: u64 = 45;
/// Regular evaluation cadence while nothing changes.
pub(super) const POLL_INTERVAL: Duration = Duration::from_secs(30);
/// A wall-clock jump this much larger than monotonic progress = we slept.
const RESUME_JUMP: Duration = Duration::from_secs(60);

fn audio_liveness_recent() -> bool {
    qbz_audio::network_throttle::state()
        .seconds_since_download()
        .map(|secs| secs <= LIVENESS_WINDOW_SECS)
        .unwrap_or(false)
}

/// Runs the actor's tick loop until the recheck channel closes. Kept as a
/// free function (not inlined into `spawn`) so the state-machine wiring is
/// unit-testable in isolation from the `tokio::spawn` plumbing.
pub(super) async fn run(
    client: reqwest::Client,
    tx: watch::Sender<ConnectivitySnapshot>,
    mut recheck_rx: tokio::sync::mpsc::Receiver<()>,
) {
    let mut judge = ConnectivityJudge::new();
    let mut next_delay = Duration::from_millis(10); // first verdict ASAP
    let mut last_tick_wall = SystemTime::now();
    let mut last_tick_mono = Instant::now();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(next_delay) => {}
            poke = recheck_rx.recv() => {
                if poke.is_none() { return; }
                judge.reset_streak();
            }
        }

        // Suspend/resume guard: wall-clock advanced much further than
        // the monotonic clock ⇒ we slept; discard the streak.
        let wall_delta = SystemTime::now()
            .duration_since(last_tick_wall)
            .unwrap_or_default();
        let mono_delta = last_tick_mono.elapsed();
        if wall_delta > mono_delta + RESUME_JUMP {
            log::info!("[Connectivity] resume detected, resetting failure streak");
            judge.reset_streak();
        }
        last_tick_wall = SystemTime::now();
        last_tick_mono = Instant::now();

        // Layer 1: OS route signal — definitive Down.
        if has_default_route() == Some(false) {
            judge.on_no_route();
            let _ = tx.send_if_modified(|s| {
                let changed = *s != judge.snapshot();
                *s = judge.snapshot();
                changed
            });
            next_delay = Duration::from_secs(3); // cheap; watch for the route to return
            continue;
        }

        // Layer 2: passive liveness — definitive Up, zero traffic.
        if audio_liveness_recent() {
            judge.on_liveness();
            let _ = tx.send_if_modified(|s| {
                let changed = *s != judge.snapshot();
                *s = judge.snapshot();
                changed
            });
            next_delay = POLL_INTERVAL;
            continue;
        }

        // Layer 3: probe + hysteresis.
        let outcome = probe_all(&client).await;
        let action = judge.on_probe(outcome, Instant::now());
        let _ = tx.send_if_modified(|s| {
            let changed = *s != judge.snapshot();
            *s = judge.snapshot();
            changed
        });
        next_delay = match action {
            JudgeAction::ConfirmAfter(delay) => delay,
            JudgeAction::Idle => POLL_INTERVAL,
        };
    }
}
