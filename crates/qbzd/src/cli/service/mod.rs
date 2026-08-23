// crates/qbzd/src/cli/service/ — `qbzd service [systemd|openrc|runit]`, a
// pure-local generator (no daemon, like `completions`) that prints a ready-to-
// install service definition for the host's init system.
//
// systemd is the standard and ships a user unit already; this generator also
// covers the inits the standard packaging can't: OpenRC (the owner's) and runit.
// The one thing those get wrong by default is the AUDIO ENVIRONMENT — a
// system-level service drops to a user but loses that user's session env, so
// PipeWire/Pulse (via `XDG_RUNTIME_DIR`) and the config/token roots (via `HOME`)
// go missing. Every non-user template sets both explicitly, resolved for the
// target user at generation time (`getent`/`id`), so the daemon finds the same
// audio stack it would in an interactive session. (An ALSA-direct/bit-perfect
// setup doesn't need `XDG_RUNTIME_DIR` at all — it's harmless there and correct
// for the PipeWire case.)
mod hints;
mod target;
mod templates;

use target::{detect_init, resolve};
use templates::{openrc, runit, systemd_system, systemd_user};
use hints::{openrc_hint, runit_hint, systemd_system_hint, systemd_user_hint};

/// `qbzd service [INIT] [--user U] [--bin PATH] [--system]`. Prints the unit to
/// stdout (pipe/redirect it into place); install steps go to stderr so stdout
/// stays clean. Exit 0, or 2 on an unknown/undetectable init.
pub fn service(init: Option<String>, user: Option<String>, bin: Option<String>, system: bool) -> i32 {
    let init = match init.map(|s| s.to_ascii_lowercase()).or_else(detect_init) {
        Some(i) => i,
        None => {
            eprintln!("error: could not detect the init system — name it explicitly");
            eprintln!("  → qbzd service systemd | openrc | runit");
            return 2;
        }
    };

    let t = resolve(user, bin);
    let (file, hint) = match init.as_str() {
        "systemd" if system => (systemd_system(&t), systemd_system_hint()),
        "systemd" => (systemd_user(&t), systemd_user_hint()),
        "openrc" => (openrc(&t), openrc_hint()),
        "runit" => (runit(&t), runit_hint()),
        other => {
            eprintln!("error: unknown init system '{other}'");
            eprintln!("  → systemd | openrc | runit");
            return 2;
        }
    };

    print!("{file}");
    eprint!("{hint}");
    0
}
