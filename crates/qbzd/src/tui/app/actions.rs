// crates/qbzd/src/tui/app/actions.rs — the non-save immediate actions: device
// enumeration, token login, logout, and the T12 import/export spawns.

use qbz_audio::AudioBackendType;

use crate::login;
use crate::tui::screens::account::AuthSnapshot;
use crate::tui::strings as s;

use super::messages_worker::{Active, Msg, Overlay};
use super::state::App;
use super::worker_fns::enumerate_devices;
use super::worker_import::{apply_import, export_bundle};
use super::worker_import_plan::plan_import;

impl App {
    pub(super) fn spawn_devices(&mut self, backend: AudioBackendType) {
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::Devices(enumerate_devices(backend)));
        });
    }

    pub(super) fn spawn_token_login(&mut self, token: String) {
        self.busy = Some(s::ACCOUNT_VALIDATING.to_string());
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            let res = login::login_with_token_arg(&roots, &token)
                .await
                .map(|session| (session.email, Some(session.subscription_label)))
                .map_err(|e| e.to_string());
            let _ = tx.send(Msg::TokenLogin(res));
        });
    }

    pub(super) fn do_logout(&mut self) {
        match login::logout(&self.roots) {
            Ok(_) => {
                self.auth = AuthSnapshot {
                    logged_in: false,
                    email: None,
                    plan: None,
                    cred_file_present: false,
                };
                if let Active::Account(s) = &mut self.active {
                    s.set_auth(self.auth.clone());
                }
                self.overlay = Overlay::Result {
                    title: s::ACCOUNT_TITLE.to_string(),
                    lines: vec!["logged out".to_string()],
                };
            }
            Err(e) => {
                self.overlay = Overlay::Result {
                    title: s::ACCOUNT_TITLE.to_string(),
                    lines: vec![e.to_string()],
                };
            }
        }
    }

    pub(super) fn spawn_import_plan(&mut self, path: String) {
        self.busy = Some("reading bundle…".to_string());
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::ImportPlanned(plan_import(&roots, &path).map(Box::new)));
        });
    }

    pub(super) fn spawn_import_apply(&mut self) {
        let ctx = match &self.active {
            Active::Bundle(s) => s.apply_context(),
            _ => None,
        };
        let Some((bundle, target, live, mut opts, choice, with_auth)) = ctx else {
            return;
        };
        self.busy = Some("applying import…".to_string());
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        self.handle.spawn(async move {
            opts.include_auth = with_auth;
            let msg = apply_import(&roots, bundle, target, live, opts, choice).await;
            let _ = tx.send(msg);
        });
    }

    pub(super) fn spawn_export(&mut self, dest: String, include_auth: bool) {
        self.busy = Some("exporting…".to_string());
        let roots = self.roots.clone();
        let tx = self.tx.clone();
        self.handle.spawn_blocking(move || {
            let _ = tx.send(Msg::Exported(export_bundle(&roots, &dest, include_auth)));
        });
    }
}
