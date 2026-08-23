// crates/qbzd/src/tui/app/keys.rs — the top-level key handler: overlays
// capture first, then the FB3 dual-focus nav-intent table resolves the rest
// (screen dispatch itself lives in `keys_dispatch.rs`).

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::messages::{LoopCmd, Screen};
use super::messages_worker::Overlay;
use super::nav_classify::{classify_key, NavIntent};
use super::state::App;

impl App {
    pub fn on_key(&mut self, key: KeyEvent) -> LoopCmd {
        if self.busy.is_some() {
            return LoopCmd::None; // §5.5: input parked while a worker runs
        }
        // Overlays capture keys first.
        match &self.overlay {
            Overlay::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                    self.overlay = Overlay::None;
                }
                return LoopCmd::None;
            }
            Overlay::Result { .. } => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.overlay = Overlay::None;
                }
                return LoopCmd::None;
            }
            Overlay::DirtyLeave { target } => {
                let target = *target;
                match key.code {
                    KeyCode::Char('s') => {
                        self.overlay = Overlay::None;
                        self.save_active(Some(target));
                    }
                    KeyCode::Char('d') => {
                        self.overlay = Overlay::None;
                        self.apply_leave(target);
                    }
                    KeyCode::Esc => self.overlay = Overlay::None,
                    _ => {}
                }
                return LoopCmd::None;
            }
            Overlay::ConfirmAbandon => {
                match key.code {
                    // Quit the wizard: discard its transient state (a fresh
                    // WizardState) and drop focus back to the sidebar.
                    KeyCode::Char('y') | KeyCode::Enter => {
                        self.overlay = Overlay::None;
                        self.enter_screen(Screen::Wizard);
                        self.enter_nav_focus();
                    }
                    KeyCode::Esc | KeyCode::Char('n') => self.overlay = Overlay::None,
                    _ => {}
                }
                return LoopCmd::None;
            }
            Overlay::None => {}
        }

        // FB3 dual focus: resolve the key's navigation meaning purely, then
        // execute it. Content field keys (ToScreen) still flow to the active
        // screen exactly as before.
        let editing = self.active_is_editing();
        let uses_h = self.content_uses_horizontal();
        match classify_key(self.focus, key.code, editing, uses_h) {
            NavIntent::None => LoopCmd::None,
            NavIntent::MoveCursor(d) => {
                self.move_cursor(d);
                LoopCmd::None
            }
            NavIntent::ActivateCursor => {
                self.request_section(super::messages::SCREENS[self.nav_cursor]);
                LoopCmd::None
            }
            NavIntent::JumpSection(idx) => {
                self.request_section(super::messages::SCREENS[idx]);
                LoopCmd::None
            }
            NavIntent::FocusNav => {
                self.enter_nav_focus();
                LoopCmd::None
            }
            NavIntent::Quit => {
                self.leave_quit();
                LoopCmd::None
            }
            NavIntent::Help => {
                self.overlay = Overlay::Help;
                LoopCmd::None
            }
            NavIntent::ToScreen => {
                let action = self.dispatch_screen_key(key);
                self.handle_screen_action(action)
            }
        }
    }
}
