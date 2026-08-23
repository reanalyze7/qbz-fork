// crates/qbzd/src/tui/app/save.rs — the async save flow shared by every
// dirty-able screen (Audio/Playback/Network), routed through T11's
// `write_one` / the network rewriter, then a reload nudge.

use crate::tui::strings as s;

use super::messages_worker::{Active, LeaveTarget, Msg, Overlay};
use super::state::App;
use super::worker_fns::{do_reload, save_network, write_keys};

impl App {
    pub(super) fn save_active(&mut self, then_leave: Option<LeaveTarget>) {
        let (keys, network) = match &self.active {
            Active::Audio(s) => (s.save_keys(), None),
            Active::Playback(s) => (s.save_keys(), None),
            Active::Network(s) => match s.validated() {
                Ok(v) => (Vec::new(), Some(v)),
                Err(e) => {
                    self.overlay = Overlay::Result {
                        title: s::SAVE_TITLE.to_string(),
                        lines: vec![format!("cannot save: {e}")],
                    };
                    return;
                }
            },
            _ => return, // Account / Bundle / Menu never save
        };

        if keys.is_empty() && network.is_none() {
            // Nothing changed — just leave if that was the intent.
            if let Some(t) = then_leave {
                self.apply_leave(t);
            }
            return;
        }

        // The baseline is updated only on a SUCCESSFUL write (§4.2: a failed
        // store write leaves the screen dirty). Input is parked while busy, so
        // the staged form cannot change under the async save.
        self.leave_after_save = then_leave;
        self.busy = Some("saving…".to_string());
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        let is_network = network.is_some();
        self.handle.spawn(async move {
            let (write_err, reinit) = if let Some((bind, port, token)) = network {
                (save_network(&roots, &bind, port, token.as_deref()), false)
            } else {
                write_keys(&roots, &keys)
            };
            let success = write_err.is_none();
            if let Some(err) = write_err {
                // Store write failed — do not touch the daemon; report the fault.
                let _ = tx.send(Msg::Saved {
                    lines: vec![err],
                    status: None,
                    reachable: true,
                    success: false,
                });
                return;
            }
            let (lines, status, reachable) = do_reload(&roots, is_network, reinit).await;
            let _ = tx.send(Msg::Saved { lines, status, reachable, success });
        });
    }
}
