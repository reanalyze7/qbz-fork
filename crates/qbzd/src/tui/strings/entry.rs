// ============================ entry / guards ============================

/// Non-tty rejection (03 §2.4, VERBATIM). Printed to stderr, exit 2.
pub const NON_TTY_ERROR: &str = "error: 'qbzd setup' needs an interactive terminal
  → import a settings bundle:   qbzd settings import <file.qbzb>
  → set one value:              qbzd settings set <key> <value>
  → log in without the TUI:     qbzd login   (or: qbzd login --token <token>)";

/// Terminal-too-small line (03 §5.4). `w`/`h` are the current dimensions.
pub fn too_small(w: u16, h: u16) -> String {
    format!("terminal too small — 80×24 minimum (current: {w}×{h})")
}
