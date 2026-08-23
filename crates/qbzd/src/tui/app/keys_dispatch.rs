// crates/qbzd/src/tui/app/keys_dispatch.rs — routing a key to the active
// screen, then mapping its returned `ScreenAction` onto App-level effects.

use ratatui::crossterm::event::KeyEvent;

use super::messages::{LoopCmd, ScreenAction};
use super::messages_worker::{Active, Overlay};
use super::state::App;

impl App {
    pub(super) fn dispatch_screen_key(&mut self, key: KeyEvent) -> ScreenAction {
        match &mut self.active {
            Active::Account(s) => s.handle_key(key),
            Active::Audio(s) => s.handle_key(key),
            Active::Playback(s) => s.handle_key(key),
            Active::Network(s) => s.handle_key(key),
            Active::Bundle(s) => s.handle_key(key),
            Active::Wizard(s) => s.handle_key(key),
            Active::Scrobbler(s) => s.handle_key(key),
        }
    }

    pub(super) fn handle_screen_action(&mut self, action: ScreenAction) -> LoopCmd {
        match action {
            ScreenAction::Consumed => LoopCmd::None,
            ScreenAction::Save => {
                self.save_active(None);
                LoopCmd::None
            }
            ScreenAction::Back => {
                // FB3: Esc in the content returns focus to the sidebar (the
                // section stays loaded — a dirty section is still dirty).
                self.enter_nav_focus();
                LoopCmd::None
            }
            ScreenAction::RefreshDevices => {
                if let Active::Audio(s) = &self.active {
                    let backend = s.backend();
                    self.spawn_devices(backend);
                }
                LoopCmd::None
            }
            ScreenAction::LoginBrowser => LoopCmd::BrowserLogin,
            ScreenAction::LoginToken(token) => {
                self.spawn_token_login(token);
                LoopCmd::None
            }
            ScreenAction::Logout => {
                self.do_logout();
                LoopCmd::None
            }
            ScreenAction::ImportPlan(path) => {
                self.spawn_import_plan(path);
                LoopCmd::None
            }
            ScreenAction::ImportApply => {
                self.spawn_import_apply();
                LoopCmd::None
            }
            ScreenAction::Export { dest, include_auth } => {
                self.spawn_export(dest, include_auth);
                LoopCmd::None
            }
            ScreenAction::WizardProbeHealth => {
                self.spawn_wizard_health();
                LoopCmd::None
            }
            ScreenAction::WizardDetect => {
                self.spawn_wizard_detect();
                LoopCmd::None
            }
            ScreenAction::WizardGenConfigs(dacs) => {
                self.spawn_wizard_configs(dacs);
                LoopCmd::None
            }
            ScreenAction::WizardTestStart => {
                self.spawn_wizard_test(true);
                LoopCmd::None
            }
            ScreenAction::WizardTestPoll => {
                self.spawn_wizard_test(false);
                LoopCmd::None
            }
            ScreenAction::WizardAbandon => {
                self.overlay = Overlay::ConfirmAbandon;
                LoopCmd::None
            }
            ScreenAction::ScrobbleConnectLastfm => LoopCmd::ScrobbleLastfm,
            ScreenAction::ScrobbleConnectListenbrainz => LoopCmd::ScrobbleListenbrainz,
        }
    }
}
