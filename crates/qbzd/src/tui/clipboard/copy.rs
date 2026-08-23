use std::io::Write;
use std::process::{Command, Stdio};

use super::files::write_wizard_file;
use super::osc52::{base64, osc52_fits, osc52_payload};
use super::tiers::{plan_tiers, ClipEnv, Tier};

/// What a copy attempt did, so the operator is always told which tier worked.
pub struct CopyReport {
    pub tier: Tier,
    /// A one-line, human message ("copied to clipboard (OSC 52)", "clipboard
    /// unavailable — saved to <path>").
    pub detail: String,
}

/// Copy `text` to the clipboard via the best available tier. `stem` names the
/// file-fallback save. Never returns an error: the last tier is a file write,
/// and even if THAT fails the report says so rather than losing the flow.
pub fn copy(text: &str, stem: &str, env: &ClipEnv) -> CopyReport {
    // Skip the OSC 52 tier outright when the payload is too big for a
    // terminal to reliably accept — no tty write is even attempted.
    let oversized = !osc52_fits(base64(text.as_bytes()).len());
    for tier in plan_tiers(env) {
        if tier == Tier::Osc52 && oversized {
            continue; // too large for OSC 52 — go straight to the next tier
        }
        match try_tier(tier, text, stem, env.tmux) {
            Some(detail) => {
                // Osc52 always sits directly before File in plan_tiers, so
                // when it was skipped for size, name that reason instead of
                // File's generic "clipboard unavailable" (which covers the
                // headless / no-wl-copy-or-xclip case too).
                let detail = if oversized && tier == Tier::File {
                    detail.replacen("clipboard unavailable", "too large for OSC 52", 1)
                } else {
                    detail
                };
                return CopyReport { tier, detail };
            }
            None => continue,
        }
    }
    // plan_tiers always ends in File; reaching here means even the file write
    // failed. Report it — do not panic, do not lose the operator's flow.
    CopyReport {
        tier: Tier::File,
        detail: "could not copy or save the block".to_string(),
    }
}

/// Attempt one tier. `Some(detail)` on success, `None` to fall through.
fn try_tier(tier: Tier, text: &str, stem: &str, tmux: bool) -> Option<String> {
    match tier {
        Tier::Osc52 => write_osc52(text, tmux).ok().map(|()| {
            // Honest wording: the escape reached the tty, but OSC 52 is
            // one-way — plenty of terminals ignore it by default — so this
            // is NOT "copied", it's "sent, unconfirmed".
            if tmux {
                "sent via OSC 52 (tmux) — paste to confirm".to_string()
            } else {
                "sent via OSC 52 — paste to confirm".to_string()
            }
        }),
        Tier::WlCopy => pipe_to("wl-copy", &[], text)
            .then(|| "copied to clipboard (wl-copy)".to_string()),
        Tier::Xclip => pipe_to("xclip", &["-selection", "clipboard"], text)
            .then(|| "copied to clipboard (xclip)".to_string()),
        Tier::File => write_wizard_file(stem, text)
            .ok()
            .map(|p| format!("clipboard unavailable — saved to {}", p.display())),
    }
}

/// Write the OSC 52 escape to the controlling tty (`/dev/tty`) so it reaches the
/// terminal even under the ratatui alt-screen (it is an escape, not drawn text).
fn write_osc52(text: &str, tmux: bool) -> std::io::Result<()> {
    let payload = osc52_payload(text, tmux);
    let mut tty = std::fs::OpenOptions::new().write(true).open("/dev/tty")?;
    tty.write_all(payload.as_bytes())?;
    tty.flush()
}

/// Spawn `cmd args`, feed `text` on stdin, discard its output. Both `wl-copy`
/// and `xclip` fork a background holder after reading stdin, so `wait()`
/// returns promptly. `true` on spawn+write+exit success.
fn pipe_to(cmd: &str, args: &[&str], text: &str) -> bool {
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(_) => return false, // not installed → fall through
    };
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(text.as_bytes()).is_err() {
            return false;
        }
    }
    // Dropping stdin closes the pipe; the holder daemon detaches, so wait() ends.
    matches!(child.wait(), Ok(status) if status.success())
}
