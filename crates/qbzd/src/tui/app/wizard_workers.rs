// crates/qbzd/src/tui/app/wizard_workers.rs — the FB4 HiFi Wizard's worker
// spawns (health probe / DAC enumeration / config generation / test
// read-back), grouped since they're one cohesive feature slice.

use serde_json::{json, Value};

use crate::cli::client::{ApiClient, CliError};
use crate::tui::strings as s;
use crate::tui::wizard_core;

use super::messages_worker::Msg;
use super::state::App;

impl App {
    /// The heavy audio-stack health probe (shells out to systemctl/aplay/pw-dump)
    /// — never on the render thread.
    pub(super) fn spawn_wizard_health(&mut self) {
        self.busy = Some(s::WIZ_HEALTH_CHECKING.to_string());
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::WizardHealth(qbz_audio::audio_stack_health()));
        });
    }

    /// Enumerate DAC candidates via the pw-dump-robust path (blocking).
    pub(super) fn spawn_wizard_detect(&mut self) {
        self.busy = Some(s::WIZ_DETECTING.to_string());
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::WizardDacs(wizard_core::detect_blocking()));
        });
    }

    /// Re-probe rates + build the per-DAC config blocks (blocking).
    pub(super) fn spawn_wizard_configs(&mut self, dacs: Vec<(String, String)>) {
        self.busy = Some(s::WIZ_GENERATING.to_string());
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::WizardConfigs(wizard_core::gen_configs_blocking(dacs)));
        });
    }

    /// Test read-back: optionally cold-start the daemon's OWN playback of the
    /// current queue, then sample the requested rate (`/api/status`) and the
    /// DAC's REAL negotiated rate (`/proc/asound` via qbz-audio::dac_probe). The
    /// requested-vs-negotiated pair is the bit-perfect proof (N6).
    pub(super) fn spawn_wizard_test(&mut self, start_playback: bool) {
        self.busy = Some(if start_playback {
            "starting playback…".to_string()
        } else {
            "reading DAC…".to_string()
        });
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let client = ApiClient::new(None, &roots);
            let mut note = None;
            if start_playback {
                match client.post("/api/playback/play", json!({})).await {
                    Ok(_) => {
                        // Give the stream a moment to open on the DAC before the read.
                        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
                    }
                    Err(CliError::Unreachable(_)) => {
                        note = Some("daemon not reachable — start it, then try the test".to_string());
                    }
                    Err(_) => {
                        note = Some(
                            "nothing to play — queue a track or cast to the daemon first".to_string(),
                        );
                    }
                }
            }
            // Requested (what QBZ asked the daemon for), while playing.
            let requested = client.get("/api/status").await.ok().and_then(|st| {
                let sr = st.pointer("/audio/sample_rate").and_then(Value::as_u64)?;
                let bd = st.pointer("/audio/bit_depth").and_then(Value::as_u64).unwrap_or(0);
                Some((sr as u32, bd as u32))
            });
            // Negotiated (the DAC's real hardware clock).
            let negotiated = qbz_audio::negotiated_active_rate();
            let _ = tx.send(Msg::WizardTest { requested, negotiated, note });
        });
    }
}
