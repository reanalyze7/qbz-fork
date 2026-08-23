// crates/qbzd/src/tui/app/tests_focus.rs — landing focus, breadcrumb
// composition, the sidebar dirty marker, and the FB3 key-classification
// table.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::messages::Screen;
use super::nav::{breadcrumb_nodes, initial_focus, sidebar_dirty_marker, Focus};
use super::nav_classify::{classify_key, NavIntent};
use super::tests_support::bare_app;

// ---- landing (FB3): always Account; focus depends on the credential file ----

#[test]
fn first_run_lands_focus_on_content() {
    // No credential file → the operator should be able to log in immediately,
    // so the CONTENT (Account) owns focus.
    assert_eq!(initial_focus(false), Focus::Content);
}

#[test]
fn returning_user_lands_focus_on_nav() {
    // A credential file exists → land with the NAV focused so the operator
    // picks where to go (the section is still Account underneath).
    assert_eq!(initial_focus(true), Focus::Nav);
}

// ---- breadcrumb composition (max 2 levels) ----

#[test]
fn breadcrumb_is_setup_then_section_when_not_editing() {
    assert_eq!(breadcrumb_nodes("Audio", None), ("Setup", "Audio"));
}

#[test]
fn breadcrumb_is_section_then_field_when_editing() {
    assert_eq!(breadcrumb_nodes("Audio", Some("Backend")), ("Audio", "Backend"));
}

// ---- sidebar dirty marker: only the active section can show `*` ----

#[test]
fn dirty_marker_only_on_the_active_dirty_section() {
    assert!(sidebar_dirty_marker(Screen::Audio, Screen::Audio, true));
    assert!(!sidebar_dirty_marker(Screen::Audio, Screen::Audio, false));
    // A non-active section is clean by construction (leave is guarded).
    assert!(!sidebar_dirty_marker(Screen::Playback, Screen::Audio, true));
}

// ---- focus-transition table (Tab / Esc / Enter / arrows / number keys) ----

#[test]
fn nav_focus_key_table() {
    use NavIntent::*;
    let n = |code| classify_key(Focus::Nav, code, false, false);
    assert_eq!(n(KeyCode::Up), MoveCursor(-1));
    assert_eq!(n(KeyCode::Char('k')), MoveCursor(-1));
    assert_eq!(n(KeyCode::Down), MoveCursor(1));
    assert_eq!(n(KeyCode::Char('j')), MoveCursor(1));
    assert_eq!(n(KeyCode::Enter), ActivateCursor);
    assert_eq!(n(KeyCode::Right), ActivateCursor);
    assert_eq!(n(KeyCode::Tab), ActivateCursor);
    assert_eq!(n(KeyCode::Esc), Quit);
    assert_eq!(n(KeyCode::Char('q')), Quit);
    assert_eq!(n(KeyCode::Char('?')), Help);
    assert_eq!(n(KeyCode::Left), None); // no-op at the left edge
}

#[test]
fn content_focus_key_table() {
    use NavIntent::*;
    let c = |code| classify_key(Focus::Content, code, false, false);
    // Tab and (un-consumed) ← walk back to the sidebar.
    assert_eq!(c(KeyCode::Tab), FocusNav);
    assert_eq!(c(KeyCode::Left), FocusNav);
    // These belong to the screen (Esc returns Back → the App re-focuses nav).
    assert_eq!(c(KeyCode::Esc), ToScreen);
    assert_eq!(c(KeyCode::Up), ToScreen);
    assert_eq!(c(KeyCode::Enter), ToScreen);
    assert_eq!(c(KeyCode::Char('s')), ToScreen);
    assert_eq!(c(KeyCode::Right), ToScreen);
    // Global chrome.
    assert_eq!(c(KeyCode::Char('q')), Quit);
    assert_eq!(c(KeyCode::Char('?')), Help);
}

#[test]
fn left_is_consumed_by_a_horizontal_field() {
    // Audio's Buffer slider claims ← — it must NOT drop focus to the nav.
    assert_eq!(
        classify_key(Focus::Content, KeyCode::Left, false, true),
        NavIntent::ToScreen
    );
}

#[test]
fn wizard_welcome_left_drops_focus_to_nav_like_every_other_section() {
    // A fresh Wizard starts on Welcome, where nothing claims ← — it must
    // behave exactly like every non-Wizard section: ← walks focus back
    // to the sidebar instead of being swallowed by the wizard's own
    // (no-op at Welcome) step-back handling. This only exercises the
    // FocusNav path (no screen dispatch), so it never touches the
    // wizard's async health-probe worker.
    let mut app = bare_app(Screen::Wizard, Focus::Content);
    assert!(!app.content_uses_horizontal());
    app.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
    assert_eq!(app.focus, Focus::Nav);
}

#[test]
fn number_keys_jump_from_any_focus_but_not_while_editing() {
    // 1-7 jump from nav and from content (not editing).
    assert_eq!(classify_key(Focus::Nav, KeyCode::Char('3'), false, false), NavIntent::JumpSection(2));
    assert_eq!(classify_key(Focus::Content, KeyCode::Char('1'), false, false), NavIntent::JumpSection(0));
    assert_eq!(classify_key(Focus::Content, KeyCode::Char('6'), false, false), NavIntent::JumpSection(5));
    // 7 reaches the FB4 Wizard section (the seventh).
    assert_eq!(classify_key(Focus::Content, KeyCode::Char('7'), false, false), NavIntent::JumpSection(6));
    // While a field editor is open, digits are typed into it, not swallowed.
    assert_eq!(classify_key(Focus::Content, KeyCode::Char('5'), true, false), NavIntent::ToScreen);
    // ...and so is every other key while editing.
    assert_eq!(classify_key(Focus::Content, KeyCode::Tab, true, false), NavIntent::ToScreen);
    assert_eq!(classify_key(Focus::Content, KeyCode::Esc, true, false), NavIntent::ToScreen);
}
