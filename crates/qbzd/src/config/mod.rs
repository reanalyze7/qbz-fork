// crates/qbzd/src/config/ — qbzd.toml: PROCESS concerns only (D14 single-source rule).
// Engine settings (audio/playback/qconnect content) live in the stores — never here.
// QConnect startup_mode/device_name/volume_mode live SOLELY in the daemon-root
// qconnect_settings.db KV (03 §3.4/§6; helpers land in T9) — no [qconnect] table.
mod known_keys;
mod loader;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)] // Serialize: `qbzd config show --json` (T11)
#[serde(default)]
pub struct QbzdConfig {
    pub config_version: u32,
    pub data_root: Option<String>, // container override; cache root derived
    pub server: ServerCfg,
    pub log: LogCfg,
    pub mpris: MprisCfg, // documented now, inert in P0 (01 §11)
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerCfg {
    pub bind: String,
    pub port: u16,
    /// Opt-in shared secret (02 §3.1.2). Default `None` = the control plane is
    /// UNAUTHENTICATED (loopback and LAN alike). When set, every route except
    /// `GET /api/ping` requires `Authorization: Bearer <token>`; a mismatch is
    /// `401 invalid_token`. A plain config value the user writes — there is no
    /// generated file and no rotation verb (rotate = edit this + restart).
    pub token: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LogCfg {
    pub level: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MprisCfg {
    pub enabled: bool,
}

impl Default for ServerCfg {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0".into(), // LAN-first posture (FB6): Sonos/Chromecast-style
            // open renderer; the Origin shield still guards browsers and
            // `[server] token` remains the opt-in restriction for powerusers.
            port: 8182,
            token: None, // open by default (02 §3.1.2)
        }
    }
}
impl Default for LogCfg {
    fn default() -> Self {
        Self {
            level: "info".into(),
        }
    }
}
impl Default for MprisCfg {
    fn default() -> Self {
        Self { enabled: true }
    }
}
impl Default for QbzdConfig {
    fn default() -> Self {
        Self {
            config_version: 1,
            data_root: None,
            server: Default::default(),
            log: Default::default(),
            mpris: Default::default(),
        }
    }
}
