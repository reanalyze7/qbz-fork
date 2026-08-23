// crates/qbzd/src/tui/app/state_query.rs — focus/dirty/editing queries over
// the currently-active screen.

use super::messages_worker::Active;
use super::state::App;

impl App {
    pub(super) fn active_is_dirty(&self) -> bool {
        match &self.active {
            Active::Audio(s) => s.is_dirty(),
            Active::Playback(s) => s.is_dirty(),
            Active::Network(s) => s.is_dirty(),
            _ => false,
        }
    }

    pub(super) fn active_is_editing(&self) -> bool {
        match &self.active {
            Active::Account(s) => s.is_editing(),
            Active::Audio(s) => s.is_editing(),
            Active::Playback(s) => s.is_editing(),
            Active::Network(s) => s.is_editing(),
            Active::Bundle(s) => s.is_editing(),
            Active::Wizard(s) => s.is_editing(),
            Active::Scrobbler(s) => s.is_editing(),
        }
    }

    /// The breadcrumb's level-2 node (field label) when an inline edit is active.
    /// The Wizard uses it for the current STEP name (`Wizard › <step>`).
    pub(super) fn active_editing_label(&self) -> Option<&'static str> {
        match &self.active {
            Active::Account(s) => s.editing_label(),
            Active::Audio(s) => s.editing_label(),
            Active::Playback(s) => s.editing_label(),
            Active::Network(s) => s.editing_label(),
            Active::Bundle(s) => s.editing_label(),
            Active::Wizard(s) => s.editing_label(),
            Active::Scrobbler(s) => s.editing_label(),
        }
    }

    /// Whether the focused content field consumes ←/→ (so ← must not drop focus).
    /// The Wizard claims ←/→ for step back/next navigation.
    pub(super) fn content_uses_horizontal(&self) -> bool {
        match &self.active {
            Active::Audio(s) => s.focused_is_buffer(),
            Active::Wizard(s) => s.claims_horizontal(),
            _ => false,
        }
    }
}
