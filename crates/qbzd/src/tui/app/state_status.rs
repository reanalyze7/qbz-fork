// crates/qbzd/src/tui/app/state_status.rs — fetching + interpreting daemon
// status (auth derivation never fabricates a name offline, §3.1).

use serde_json::Value;

use crate::tui::screens::account::AuthSnapshot;

use super::state::App;
use super::worker_fns::fetch_status;

impl App {
    pub(super) fn refresh_status(&mut self) {
        let roots = self.roots.clone();
        let body = self.handle.block_on(fetch_status(roots));
        self.reachable = body.is_some();
        self.status = body;
        self.auth = self.derive_auth();
    }

    /// Resolve auth from live status (daemon up) or credential-file presence
    /// (daemon down) — NEVER fabricating a name offline (§3.1).
    pub(super) fn derive_auth(&self) -> AuthSnapshot {
        if self.reachable {
            if let Some(st) = &self.status {
                let state = st.pointer("/auth/state").and_then(Value::as_str).unwrap_or("");
                if state == "logged_in" {
                    let id = st.pointer("/auth/user_id").and_then(Value::as_u64);
                    let plan = st
                        .pointer("/auth/subscription")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    return AuthSnapshot {
                        logged_in: true,
                        email: id.map(|i| format!("user {i}")),
                        plan,
                        cred_file_present: true,
                    };
                }
                return AuthSnapshot::default();
            }
        }
        // Offline: only report credential-file presence.
        let cred = self.roots.config.join(".qbz-oauth-token").exists();
        AuthSnapshot {
            logged_in: false,
            email: None,
            plan: None,
            cred_file_present: cred,
        }
    }
}
