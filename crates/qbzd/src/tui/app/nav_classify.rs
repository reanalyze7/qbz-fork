// crates/qbzd/src/tui/app/nav_classify.rs — the FB3 focus-transition table:
// mapping a keystroke to its pure `NavIntent`. `keys.rs` executes the intent.

use ratatui::crossterm::event::KeyCode;

use super::nav::Focus;

/// A key's navigation meaning, resolved purely from the focus state (FB3
/// focus-transition table). The impure `on_key` executes the intent (dirty
/// guards, section loads, screen dispatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NavIntent {
    /// Nav: no-op (unbound key).
    None,
    /// Nav: move the sidebar highlight by ±1 (wrapping).
    MoveCursor(isize),
    /// Nav: activate the highlighted section and focus the content.
    ActivateCursor,
    /// Any focus (not editing): jump straight to section `idx` and focus content.
    JumpSection(usize),
    /// Content: drop focus back to the nav.
    FocusNav,
    /// Nav Esc/q or content q: the quit flow (dirty-guarded).
    Quit,
    /// The help overlay.
    Help,
    /// Content: hand the key to the active screen (field navigation / edit).
    ToScreen,
}

/// Map a keystroke to its `NavIntent` (FB3). `editing` = a field editor/picker
/// is open in the content (it owns every key); `uses_horizontal` = the focused
/// content field consumes ←/→ (Audio's Buffer slider), so ← must NOT drop focus.
pub(super) fn classify_key(focus: Focus, code: KeyCode, editing: bool, uses_horizontal: bool) -> NavIntent {
    // Number keys 1-8 jump from ANY focus — but only when no field editor is
    // capturing input (a port/token/name field must receive its digits).
    if !editing {
        if let KeyCode::Char(c @ '1'..='8') = code {
            return NavIntent::JumpSection(c as usize - '1' as usize);
        }
    }
    match focus {
        Focus::Nav => match code {
            KeyCode::Up | KeyCode::Char('k') => NavIntent::MoveCursor(-1),
            KeyCode::Down | KeyCode::Char('j') => NavIntent::MoveCursor(1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Tab => NavIntent::ActivateCursor,
            KeyCode::Esc | KeyCode::Char('q') => NavIntent::Quit,
            KeyCode::Char('?') => NavIntent::Help,
            _ => NavIntent::None,
        },
        Focus::Content => {
            // An open editor owns the keyboard (Esc cancels the edit, digits type,
            // etc.) — nothing is intercepted for focus changes.
            if editing {
                return NavIntent::ToScreen;
            }
            match code {
                KeyCode::Tab => NavIntent::FocusNav,
                // ← walks left toward the sidebar unless the field claims it.
                KeyCode::Left if !uses_horizontal => NavIntent::FocusNav,
                KeyCode::Char('?') => NavIntent::Help,
                KeyCode::Char('q') => NavIntent::Quit,
                // Everything else (↑↓, s, r, /, Enter, →, Esc→Back) is the
                // screen's. Esc returns `ScreenAction::Back`, which the App maps
                // to FocusNav — so Esc in content also lands on the sidebar.
                _ => NavIntent::ToScreen,
            }
        }
    }
}
