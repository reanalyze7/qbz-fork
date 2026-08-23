//! Shortcut-string grammar (port of eventToShortcut / formatShortcutDisplay).

use i_slint_backend_winit::winit::keyboard::{Key, NamedKey};

/// Normalize a winit logical key to a canonical key token (the part after the
/// modifiers). Returns `None` for bare modifier presses and unrepresentable
/// keys. Letter/symbol casing is taken from winit verbatim (winit already
/// reports the shifted glyph, so Shift+s → "S", Shift+/ → "?").
pub fn token_from_key(key: &Key) -> Option<String> {
    match key {
        Key::Named(NamedKey::Space) => Some("Space".into()),
        Key::Named(NamedKey::ArrowLeft) => Some("ArrowLeft".into()),
        Key::Named(NamedKey::ArrowRight) => Some("ArrowRight".into()),
        Key::Named(NamedKey::ArrowUp) => Some("ArrowUp".into()),
        Key::Named(NamedKey::ArrowDown) => Some("ArrowDown".into()),
        Key::Named(NamedKey::Escape) => Some("Escape".into()),
        Key::Named(NamedKey::Enter) => Some("Enter".into()),
        Key::Named(NamedKey::Tab) => Some("Tab".into()),
        Key::Named(NamedKey::Backspace) => Some("Backspace".into()),
        Key::Named(NamedKey::Delete) => Some("Delete".into()),
        Key::Character(s) => {
            let t = s.as_str();
            if t.chars().count() != 1 {
                return None;
            }
            Some(t.to_string())
        }
        _ => None,
    }
}

/// Build the canonical shortcut string from modifiers + a key token.
pub fn shortcut_from_parts(ctrl: bool, alt: bool, shift: bool, token: &str) -> Option<String> {
    if token.is_empty() {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    if ctrl {
        parts.push("Ctrl".into());
    }
    if alt {
        parts.push("Alt".into());
    }
    // Shift is emitted only for letters, digits, and named (multi-char) keys.
    let is_named = token.chars().count() > 1;
    let single = token.chars().next().unwrap();
    let is_letter = !is_named && single.is_ascii_alphabetic();
    let is_digit = !is_named && single.is_ascii_digit();
    if shift && (is_named || is_letter || is_digit) {
        parts.push("Shift".into());
    }
    parts.push(token.to_string());
    Some(parts.join("+"))
}

const KEY_DISPLAY: &[(&str, &str)] = &[
    // Solid triangles, not the thin Unicode arrows (←→↑↓) whose heads were
    // nearly invisible at keycap size — these render a clear, filled arrowhead.
    ("ArrowLeft", "◀"),
    ("ArrowRight", "▶"),
    ("ArrowUp", "▲"),
    ("ArrowDown", "▼"),
    ("Space", "Space"),
    ("Escape", "Esc"),
    ("Enter", "↵"),
    ("Backspace", "⌫"),
    ("Delete", "Del"),
    ("Tab", "Tab"),
];

/// Format a shortcut string for display (port of `formatShortcutDisplay`).
/// macOS uses ⌘⌥⇧ glyphs joined by spaces; elsewhere "Ctrl + …".
pub fn format_display(shortcut: &str) -> String {
    if shortcut.is_empty() {
        return String::new();
    }
    let (mut ctrl, mut alt, mut shift) = (false, false, false);
    let mut key = "";
    for part in shortcut.split('+') {
        match part {
            "Ctrl" => ctrl = true,
            "Alt" => alt = true,
            "Shift" => shift = true,
            other => key = other,
        }
    }
    let mac = cfg!(target_os = "macos");
    let mut out: Vec<String> = Vec::new();
    if ctrl {
        out.push(if mac { "⌘" } else { "Ctrl" }.into());
    }
    if alt {
        out.push(if mac { "⌥" } else { "Alt" }.into());
    }
    if shift {
        out.push(if mac { "⇧" } else { "Shift" }.into());
    }
    let disp = KEY_DISPLAY
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| key.to_uppercase());
    out.push(disp);
    out.join(if mac { " " } else { " + " })
}
