// ============================ error voice (02 §1.4) ============================
// Verbatim §1.4 / §2.2 / §6.3 copy — "modulo interpolated values". Every message
// ends in one to three `→` fix lines; changing the wording is a spec violation.

/// Daemon unreachable — exit 3 (02 §1.4). `host` is the target `ip:port`.
pub fn daemon_down(host: &str) -> String {
    format!(
        "error: daemon not reachable at {host}
  → is it running?    systemctl --user status qbzd
  → just installed?   systemctl --user enable --now qbzd
  → different host?   qbzd --host <ip>:<port> ...  or  export QBZD_HOST=<ip>:<port>"
    )
}

/// Daemon up but not logged in — exit 4 (02 §1.4). The down-vs-unhealthy
/// distinction: the daemon answered, the Qobuz session is what's missing.
/// Rendered by `CliError::NeedsAuth`'s `Display` — hit whenever `now`/`play`/
/// `toggle`/`next`/`prev` get a 409 `needs_auth` from a NeedsAuth daemon;
/// `status` renders the composite block instead.
pub fn daemon_up_needs_auth() -> String {
    "error: daemon is running but not logged in to Qobuz
  → log in:           qbzd login
  → have a bundle?    qbzd settings import qbz-settings-20260714.qbzb --include-auth"
        .to_string()
}

/// Linger-off warning (02 §1.4; NOT an error) — printed by `qbzd status` on the
/// daemon box when `loginctl show-user $USER -p Linger` reports `Linger=no`.
pub fn linger_off(user: &str) -> String {
    format!(
        "warning: linger is off for user '{user}' — the daemon stops when you log out
  → keep it running:  sudo loginctl enable-linger {user}"
    )
}

/// Volume fixed under DSD-direct — exit 5 (02 §1.4, verbatim). Consumed by
/// `error_from_envelope` (cli/client.rs) for the `volume_fixed_dsd` code, so
/// the `volume`/`mute` verbs print this exact block instead of the server's
/// short envelope message.
pub fn volume_fixed_dsd() -> String {
    "error: volume is fixed in DSD-direct mode (bit-perfect passthrough)
  → to get software volume, set DSD mode to \"convert\":  qbzd setup  (Audio screen)"
        .to_string()
}

/// Seek unsupported under DSD-direct — exit 5. No verbatim block is given for
/// seek specifically; 02 §2.2 says it is "the same error-voice family as
/// §1.4 volume copy", so this mirrors `volume_fixed_dsd`'s structure/wording.
/// Consumed by `error_from_envelope` for the `seek_unsupported_dsd` code.
pub fn seek_unsupported_dsd() -> String {
    "error: seek is unsupported in DSD-direct mode (bit-perfect passthrough)
  → to seek, set DSD mode to \"convert\":  qbzd setup  (Audio screen)"
        .to_string()
}

/// Foreign occupant on the control port that is NOT qbzd — printed by the daemon
/// at boot step 5 (02 §2.2, verbatim). `port` is interpolated.
pub fn port_in_use(port: u16) -> String {
    format!(
        "error: port {port} is in use by another process (not qbzd)
  → change the port:  edit [server].port in ~/.config/qbzd/qbzd.toml"
    )
}

/// A DIFFERENT qbzd already answering on the port while our instance lock is on
/// another data root (a stale foreign root). Boot step 5 (02 §8.1-5).
pub fn foreign_qbzd(addr: &str) -> String {
    format!(
        "error: another qbzd is already answering on {addr} (the instance lock said this root is free — stale foreign root?)
  → find it:          ss -ltnp | grep {addr}
  → or change the port:  edit [server].port in ~/.config/qbzd/qbzd.toml"
    )
}

/// LAN-first posture note (FB6, successor to the old 02 §6.3 LAN-exposure
/// warning) — one INFO line logged by the daemon at boot when the control API
/// is NOT loopback-only. Since FB6 the default bind is `0.0.0.0`, so this
/// fires on every default boot; it is deliberately informational, not a
/// `warning:` — an open LAN renderer (Sonos/Chromecast posture) is the
/// intended default, the Origin shield already guards browsers, and this line
/// just orients the operator toward the two ways to restrict it further.
/// `addr` is the bound `ip:port`.
pub fn lan_posture_note(addr: &str) -> String {
    format!(
        "control plane listening on {addr} — anyone on your network can control playback (set [server] bind = \"127.0.0.1\" or [server] token in qbzd.toml to restrict)"
    )
}

/// Version skew — daemon and CLI run different bin semvers (02 §1.6). A warning:
/// `status` still renders. `daemon`/`cli` are the two `version` strings.
pub fn version_skew(daemon: &str, cli: &str) -> String {
    format!(
        "warning: daemon runs {daemon}, this CLI is {cli}
  → restart the daemon:  systemctl --user restart qbzd"
    )
}

/// Breaking api_version skew (02 §1.6) — the verb refuses politely, exit 1.
pub fn api_version_skew(daemon: u32, cli: u32) -> String {
    format!(
        "error: daemon speaks api v{daemon}, this CLI speaks v{cli} — update so both ends run the same package version"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lan_posture_note_renders_the_verbatim_copy() {
        // FB6 (successor to the old LAN-exposure warning) — one INFO line logged
        // by the daemon at boot when the control API is NOT loopback-only. This
        // test pins the exact wording so it cannot drift from the spec.
        let rendered = lan_posture_note("0.0.0.0:6789");
        assert!(
            rendered.contains("control plane listening on 0.0.0.0:6789"),
            "{rendered}"
        );
        assert!(
            rendered.contains("anyone on your network can control playback"),
            "{rendered}"
        );
        assert!(rendered.contains("bind = \"127.0.0.1\""), "{rendered}");
        assert!(
            rendered.contains("token in qbzd.toml to restrict"),
            "{rendered}"
        );
    }
}
