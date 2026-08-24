// crates/qbzd/src/tui/clipboard/ — SSH-first tiered clipboard for the wizard's
// config blocks (FB4). The daemon's primary operator drives it over SSH, so
// OSC 52 (a terminal escape the SSH client's terminal honours) leads there;
// on a local session the native tools (`wl-copy`/`xclip`) lead because local
// terminals often don't honour OSC 52. Every path ends at a file save so a copy
// NEVER errors out of the flow — the operator is always told which tier worked.
//
// Two pure, unit-tested pieces: `osc52_payload` (base64 + optional tmux
// passthrough wrapping) and `plan_tiers` (env-driven ordering).
mod copy;
mod files;
mod osc52;
mod tiers;

pub use copy::copy;
pub use files::write_wizard_file;
pub use tiers::{ClipEnv, Tier};
