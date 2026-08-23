// crates/qbzd/src/tui/app/worker_results.rs — draining the worker-result
// channel and applying Device/Save/TokenLogin results (the Bundle/Wizard
// results are handled in `worker_results_bundle.rs`).

use crate::tui::screens::account::AuthSnapshot;
use crate::tui::strings as s;

use super::messages_worker::{Active, Msg, Overlay};
use super::state::App;

impl App {
    pub fn drain_worker(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.on_msg(msg);
        }
    }

    pub(super) fn on_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Devices(result) => {
                if let Active::Audio(s) = &mut self.active {
                    s.set_devices(result);
                }
            }
            Msg::Saved { lines, status, reachable, success } => {
                self.busy = None;
                self.reachable = reachable;
                if status.is_some() {
                    self.status = status;
                }
                self.overlay = Overlay::Result {
                    title: s::SAVE_TITLE.to_string(),
                    lines,
                };
                if success {
                    // §4.1: the staged form becomes the baseline (dirty clears).
                    match &mut self.active {
                        Active::Audio(sc) => sc.mark_saved(),
                        Active::Playback(sc) => sc.mark_saved(),
                        Active::Network(sc) => sc.mark_saved(),
                        _ => {}
                    }
                    // Dirty-leave "Save" → leave once the save landed (§4.1).
                    if let Some(target) = self.leave_after_save.take() {
                        self.apply_leave(target);
                    }
                } else {
                    // §4.2: a failed write leaves the screen dirty; do not leave.
                    self.leave_after_save = None;
                }
            }
            Msg::TokenLogin(result) => {
                self.busy = None;
                match result {
                    Ok((email, plan)) => {
                        self.auth = AuthSnapshot {
                            logged_in: true,
                            email: Some(email.clone()),
                            plan: plan.clone(),
                            cred_file_present: true,
                        };
                        if let Active::Account(st) = &mut self.active {
                            st.set_auth(self.auth.clone());
                        }
                        self.overlay = Overlay::Result {
                            title: s::ACCOUNT_TITLE.to_string(),
                            lines: vec![s::account_logged_in(&email)],
                        };
                    }
                    Err(e) => {
                        self.overlay = Overlay::Result {
                            title: s::ACCOUNT_TITLE.to_string(),
                            lines: e.lines().map(str::to_string).collect(),
                        };
                    }
                }
            }
            other => self.on_msg_bundle_or_wizard(other),
        }
    }
}
