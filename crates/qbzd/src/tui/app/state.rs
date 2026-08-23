// crates/qbzd/src/tui/app/state.rs — the `App` struct definition +
// construction. Every other method lives in sibling `state_*.rs`/`keys*.rs`/
// `save.rs`/`actions.rs`/`worker_*.rs`/`draw*.rs` files as further `impl App`
// blocks — fields are `pub(super)` so they stay reachable throughout the
// `app` module subtree without being crate-wide public.

use std::sync::mpsc::{Receiver, Sender};

use serde_json::Value;
use tokio::runtime::Handle;

use crate::paths::ProfileRoots;
use crate::tui::screens::account::{AccountState, AuthSnapshot};

use super::messages::Screen;
use super::messages_worker::{Active, LeaveTarget, Msg, Overlay};
use super::nav::{cred_file_path, initial_focus, Focus};

pub struct App {
    pub(super) roots: ProfileRoots,
    pub(super) handle: Handle,
    pub(super) tx: Sender<Msg>,
    pub(super) rx: Receiver<Msg>,

    pub(super) active: Active,
    /// Which section `active` currently holds (its state is loaded). The sidebar
    /// marks it `▸` + accent; the breadcrumb names it.
    pub(super) active_section: Screen,
    /// The sidebar highlight while `focus == Nav`. Reset to `active_section` on
    /// every entry into nav focus; moving it does NO I/O (it only re-points the
    /// highlight — a section loads only on activation, §5.5).
    pub(super) nav_cursor: usize,
    /// Which pane owns the keyboard (FB3 dual focus).
    pub(super) focus: Focus,

    pub(super) status: Option<Value>,
    pub(super) reachable: bool,
    pub(super) auth: AuthSnapshot,

    pub(super) overlay: Overlay,
    pub(super) busy: Option<String>,
    pub busy_tick: u64,
    pub(super) should_quit: bool,
    /// Set when a save was requested from the dirty-leave modal — the leave
    /// happens once the save succeeds (§4.1 Save/Discard/Stay → Save then leave).
    pub(super) leave_after_save: Option<LeaveTarget>,
}

impl App {
    pub fn new(roots: ProfileRoots, handle: Handle) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut app = App {
            roots: roots.clone(),
            handle,
            tx,
            rx,
            active: Active::Account(AccountState::new(AuthSnapshot::default())),
            active_section: Screen::Account,
            nav_cursor: 0,
            focus: Focus::Nav,
            status: None,
            reachable: false,
            auth: AuthSnapshot::default(),
            overlay: Overlay::None,
            busy: None,
            busy_tick: 0,
            should_quit: false,
            leave_after_save: None,
        };
        // Landing (FB3): always the Account section; focus depends on whether a
        // credential file exists (first-run → content, ready to log in).
        let cred_file_exists = cred_file_path(&roots.config).exists();
        app.enter_screen(Screen::Account);
        app.focus = initial_focus(cred_file_exists);
        app
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn busy(&self) -> bool {
        self.busy.is_some()
    }

    pub fn roots(&self) -> &ProfileRoots {
        &self.roots
    }
}
