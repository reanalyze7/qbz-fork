/// One clipboard mechanism, in preference groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// OSC 52 escape written to the controlling tty — works across SSH.
    Osc52,
    /// `wl-copy` (Wayland).
    WlCopy,
    /// `xclip -selection clipboard` (X11).
    Xclip,
    /// Guaranteed fallback: write to `~/qbzd-wizard/<name>.conf`.
    File,
}

impl Tier {
    /// The per-block flash shown after a copy/save attempt. OSC 52 is
    /// one-way — the local terminal may silently ignore the escape, and
    /// there is no ack to confirm it landed — so its flash says "sent …
    /// paste to confirm" rather than claiming a completed copy. wl-copy and
    /// xclip keep the checkmark (they're locally verifiable-ish: the process
    /// exited 0 having read the pipe); File keeps naming the artifact.
    pub fn short_label(self) -> &'static str {
        match self {
            Tier::Osc52 => "sent via OSC 52 — paste to confirm",
            Tier::WlCopy => "copied ✓ (wl-copy)",
            Tier::Xclip => "copied ✓ (xclip)",
            Tier::File => "copied ✓ (saved to file)",
        }
    }
}

/// The clipboard-relevant environment, sampled once. Pure input to `plan_tiers`
/// so the ordering is testable without touching the real environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipEnv {
    pub ssh: bool,
    pub tmux: bool,
    pub wayland: bool,
    pub x11: bool,
}

impl ClipEnv {
    /// Read the live environment: SSH_TTY/SSH_CONNECTION, TMUX, WAYLAND_DISPLAY,
    /// DISPLAY.
    pub fn from_env() -> Self {
        let has = |k: &str| std::env::var_os(k).map(|v| !v.is_empty()).unwrap_or(false);
        ClipEnv {
            ssh: has("SSH_TTY") || has("SSH_CONNECTION"),
            tmux: has("TMUX"),
            wayland: has("WAYLAND_DISPLAY"),
            x11: has("DISPLAY"),
        }
    }
}

/// Ordered tiers to attempt, SSH-first. Remote (SSH/tmux) leads with OSC 52
/// since it is the one that survives the hop; a local session leads with the
/// native tool for the running display server. Always ends at `File` so a copy
/// can never fail out of the flow.
pub fn plan_tiers(env: &ClipEnv) -> Vec<Tier> {
    let remote = env.ssh || env.tmux;
    let mut tiers = Vec::new();
    if remote {
        tiers.push(Tier::Osc52);
    } else {
        if env.wayland {
            tiers.push(Tier::WlCopy);
        }
        if env.x11 {
            tiers.push(Tier::Xclip);
        }
        tiers.push(Tier::Osc52);
    }
    tiers.push(Tier::File);
    tiers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_tiers_is_ssh_first_remote() {
        let ssh = ClipEnv { ssh: true, tmux: false, wayland: true, x11: true };
        assert_eq!(plan_tiers(&ssh), vec![Tier::Osc52, Tier::File]);
        let tmux = ClipEnv { ssh: false, tmux: true, wayland: false, x11: false };
        assert_eq!(plan_tiers(&tmux), vec![Tier::Osc52, Tier::File]);
    }

    #[test]
    fn plan_tiers_prefers_native_tools_locally() {
        let wayland = ClipEnv { ssh: false, tmux: false, wayland: true, x11: true };
        assert_eq!(plan_tiers(&wayland), vec![Tier::WlCopy, Tier::Xclip, Tier::Osc52, Tier::File]);
        let x11 = ClipEnv { ssh: false, tmux: false, wayland: false, x11: true };
        assert_eq!(plan_tiers(&x11), vec![Tier::Xclip, Tier::Osc52, Tier::File]);
        let headless = ClipEnv { ssh: false, tmux: false, wayland: false, x11: false };
        assert_eq!(plan_tiers(&headless), vec![Tier::Osc52, Tier::File]);
    }

    #[test]
    fn plan_tiers_always_ends_in_file() {
        for ssh in [false, true] {
            for tmux in [false, true] {
                for wayland in [false, true] {
                    for x11 in [false, true] {
                        let env = ClipEnv { ssh, tmux, wayland, x11 };
                        assert_eq!(*plan_tiers(&env).last().unwrap(), Tier::File);
                    }
                }
            }
        }
    }
}
