// crates/qbzd/src/tui/app/worker_results_bundle.rs — applying the T12
// import/export results and the FB4 HiFi Wizard's worker results.

use crate::tui::strings as s;

use super::messages_worker::{Active, Msg, Overlay};
use super::state::App;

impl App {
    pub(super) fn on_msg_bundle_or_wizard(&mut self, msg: Msg) {
        match msg {
            Msg::ImportPlanned(result) => {
                self.busy = None;
                match result {
                    Ok(pending) => {
                        if let Active::Bundle(s) = &mut self.active {
                            s.set_plan(*pending);
                        }
                    }
                    Err(e) => {
                        self.overlay = Overlay::Result {
                            title: s::BUNDLE_TITLE.to_string(),
                            lines: e.lines().map(str::to_string).collect(),
                        };
                    }
                }
            }
            Msg::ImportApplied { lines, status, reachable } => {
                self.busy = None;
                self.reachable = reachable;
                if status.is_some() {
                    self.status = status;
                }
                if let Active::Bundle(s) = &mut self.active {
                    s.clear_pending();
                }
                // A bundle may have logged us in — refresh auth.
                self.auth = self.derive_auth();
                self.overlay = Overlay::Result {
                    title: s::BUNDLE_TITLE.to_string(),
                    lines,
                };
            }
            Msg::Exported(result) => {
                self.busy = None;
                let lines = match result {
                    Ok(lines) => lines,
                    Err(e) => e.lines().map(str::to_string).collect(),
                };
                self.overlay = Overlay::Result {
                    title: s::BUNDLE_TITLE.to_string(),
                    lines,
                };
            }
            Msg::WizardHealth(health) => {
                self.busy = None;
                if let Active::Wizard(w) = &mut self.active {
                    w.set_health(health);
                }
            }
            Msg::WizardDacs(dacs) => {
                self.busy = None;
                if let Active::Wizard(w) = &mut self.active {
                    w.set_candidates(dacs);
                }
            }
            Msg::WizardConfigs(configs) => {
                self.busy = None;
                if let Active::Wizard(w) = &mut self.active {
                    w.set_configs(configs);
                }
            }
            Msg::WizardTest { requested, negotiated, note } => {
                self.busy = None;
                if let Active::Wizard(w) = &mut self.active {
                    w.set_test_result(requested, negotiated, note);
                }
            }
            Msg::Devices(_) | Msg::Saved { .. } | Msg::TokenLogin(_) => {
                unreachable!("handled in worker_results.rs::on_msg")
            }
        }
    }
}
