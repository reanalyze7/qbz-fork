// crates/qbzd/src/tui/app/state_nav.rs — section entry/switching and the
// dirty-save guard's departure logic (the dirty/editing/horizontal queries
// live in `state_query.rs`).

use qbz_app::settings::daemon_prefs;
use qbz_app::settings::playback::PlaybackPreferencesStore;

use crate::config::QbzdConfig;
use crate::tui::screens::account::AccountState;
use crate::tui::screens::audio::AudioState;
use crate::tui::screens::bundle::BundleState;
use crate::tui::screens::network::NetworkState;
use crate::tui::screens::playback::PlaybackState;
use crate::tui::screens::scrobbler::ScrobblerState;
use crate::tui::screens::wizard::WizardState;

use super::messages::Screen;
use super::messages_worker::{Active, LeaveTarget, Overlay};
use super::nav::{section_index, Focus};
use super::state::App;
use super::worker_fns::load_audio;
use super::worker_fns_ext::desktop_profile_present;

impl App {
    /// Load a section into the content frame (a §5.5 "screen entry": disk reads
    /// + a fresh daemon-status fetch happen here, never on a keystroke). Sets the
    /// active section and syncs the sidebar cursor to it.
    pub(super) fn enter_screen(&mut self, screen: Screen) {
        self.refresh_status();
        self.active_section = screen;
        self.nav_cursor = section_index(screen);
        self.active = match screen {
            Screen::Account => Active::Account(AccountState::new(self.auth.clone())),
            Screen::Audio => {
                let audio = load_audio(&self.roots);
                let mut st = AudioState::new(&audio);
                st.start_scan();
                let backend = st.backend();
                self.spawn_devices(backend);
                Active::Audio(st)
            }
            Screen::Playback => {
                let audio = load_audio(&self.roots);
                let playback = PlaybackPreferencesStore::new_at(&self.roots.data)
                    .and_then(|s| s.get_preferences())
                    .unwrap_or_default();
                let dp = daemon_prefs::load_at(&self.roots.data);
                Active::Playback(PlaybackState::new(&dp.streaming_quality, dp.mpris_enabled, &audio, &playback))
            }
            Screen::Network => {
                let (cfg, warns) = QbzdConfig::load(&self.roots.config.join("qbzd.toml"))
                    .unwrap_or_else(|_| (QbzdConfig::default(), Vec::new()));
                Active::Network(NetworkState::new(&cfg, warns))
            }
            Screen::Bundle => Active::Bundle(BundleState::new(desktop_profile_present())),
            Screen::Wizard => Active::Wizard(WizardState::new()),
            Screen::Scrobbler => Active::Scrobbler(ScrobblerState::new(&self.roots)),
        };
    }

    /// Request a switch to `target` (sidebar activation / number key). Same
    /// section → just move focus to the content (no reload — §5.5). Different
    /// section → the dirty guard fires (Save/Discard/Stay) before the load.
    pub(super) fn request_section(&mut self, target: Screen) {
        if target == self.active_section {
            self.focus = Focus::Content;
            return;
        }
        if self.active_is_dirty() {
            self.overlay = Overlay::DirtyLeave { target: LeaveTarget::Section(target) };
            return;
        }
        self.enter_screen(target);
        self.focus = Focus::Content;
    }

    /// The quit flow, dirty-guarded (Esc/q in nav, q in content). A dirty active
    /// section opens the Save/Discard/Stay modal targeting `Quit`.
    pub(super) fn leave_quit(&mut self) {
        if self.active_is_dirty() {
            self.overlay = Overlay::DirtyLeave { target: LeaveTarget::Quit };
        } else {
            self.should_quit = true;
        }
    }

    /// Move focus to the sidebar (content → nav), re-seating the highlight on the
    /// active section.
    pub(super) fn enter_nav_focus(&mut self) {
        self.focus = Focus::Nav;
        self.nav_cursor = section_index(self.active_section);
    }

    pub(super) fn move_cursor(&mut self, delta: isize) {
        let n = super::messages::SCREENS.len() as isize;
        self.nav_cursor = (self.nav_cursor as isize + delta).rem_euclid(n) as usize;
    }

    /// Execute a dirty-guarded departure once the guard is cleared (or absent).
    pub(super) fn apply_leave(&mut self, target: LeaveTarget) {
        match target {
            LeaveTarget::Section(screen) => {
                self.enter_screen(screen);
                self.focus = Focus::Content;
            }
            LeaveTarget::Quit => self.should_quit = true,
        }
    }
}
