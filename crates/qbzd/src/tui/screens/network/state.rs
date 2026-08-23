use std::net::IpAddr;

use crate::config::QbzdConfig;
use crate::tui::strings as s;
use crate::tui::widgets::TextInput;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Staged {
    pub(super) bind: String,
    pub(super) port: String,
    pub(super) token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NField {
    Bind,
    Port,
    Token,
}

pub(super) const FIELDS: [NField; 3] = [NField::Bind, NField::Port, NField::Token];

pub struct NetworkState {
    pub(super) baseline: Staged,
    pub(super) staged: Staged,
    pub(super) focus: usize,
    pub(super) editor: Option<(NField, TextInput)>,
    /// Unrecognized qbzd.toml keys (named in the pre-save warning, §3.5).
    pub(super) unknown_keys: Vec<String>,
}

impl NetworkState {
    pub fn new(cfg: &QbzdConfig, unknown_keys: Vec<String>) -> Self {
        let staged = Staged {
            bind: cfg.server.bind.clone(),
            port: cfg.server.port.to_string(),
            token: cfg.server.token.clone().unwrap_or_default(),
        };
        Self {
            baseline: staged.clone(),
            staged,
            focus: 0,
            editor: None,
            unknown_keys,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.staged != self.baseline
    }
    pub fn is_editing(&self) -> bool {
        self.editor.is_some()
    }
    pub fn mark_saved(&mut self) {
        self.baseline = self.staged.clone();
    }

    /// The breadcrumb's level-2 node when a field editor is active.
    pub fn editing_label(&self) -> Option<&'static str> {
        self.editor.as_ref().map(|(f, _)| match f {
            NField::Bind => s::N_BIND,
            NField::Port => s::N_PORT,
            NField::Token => s::N_TOKEN,
        })
    }

    /// Validated (bind, port, token) ready for the TOML rewrite, or field errors.
    pub fn validated(&self) -> Result<(String, u16, Option<String>), String> {
        if self.staged.bind.parse::<IpAddr>().is_err() {
            return Err(s::N_BAD_IP.to_string());
        }
        let port: u16 = match self.staged.port.parse() {
            Ok(p) if p >= 1 => p,
            _ => return Err(s::N_BAD_PORT.to_string()),
        };
        let token = if self.staged.token.trim().is_empty() {
            None
        } else {
            Some(self.staged.token.clone())
        };
        Ok((self.staged.bind.clone(), port, token))
    }

    /// A non-loopback bind is reachable beyond localhost (§3.5); 0.0.0.0
    /// (unspecified) binds every interface, so it warns too.
    pub(super) fn bind_is_lan(&self) -> bool {
        self.staged
            .bind
            .parse::<IpAddr>()
            .map(|ip| !ip.is_loopback())
            .unwrap_or(false)
    }

    pub(super) fn port_invalid(&self) -> bool {
        !self.staged.port.parse::<u16>().map(|p| p >= 1).unwrap_or(false)
    }
}
