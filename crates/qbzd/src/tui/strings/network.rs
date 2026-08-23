// ============================ Network (§3.5) ============================

pub const NETWORK_TITLE: &str = "Network";
/// In-screen section box title.
pub const NETWORK_SECTION: &str = "HTTP SERVER";
pub const N_BIND: &str = "Bind address";
pub const N_PORT: &str = "Port";
pub const N_TOKEN: &str = "Access token";
pub const N_TOKEN_HINT: &str = "(empty = open)";

/// LAN-first posture note shown when bind is non-loopback (§3.5, copy normative).
pub const NETWORK_LAN_POSTURE: &str = "open LAN control (Sonos/Chromecast posture) — anyone on your network can control playback\n  restrict: bind = \"127.0.0.1\" or set [server] token in qbzd.toml";

/// Restart-required copy on a bind/port/token save (§3.5).
pub const NETWORK_RESTART: &str =
    "bind/port change needs a restart — systemctl --user restart qbzd";

pub const N_BAD_IP: &str = "invalid IP address";
pub const N_BAD_PORT: &str = "port must be 1-65535";

/// Pre-save warning naming keys outside the daemon schema. Per 03 §3.5 they are
/// PRESERVED on save (a save must never destroy a released key) — only comments
/// and formatting are lost. (The brief said "drops"; 03 wins — flagged in the
/// report.) Keys are appended.
pub const N_DROP_UNKNOWN: &str =
    "note: qbzd.toml has keys outside the daemon schema — they are kept, but\n  comments and formatting are not preserved on save:";
