use std::time::Duration;

use serde_json::Value;

use crate::paths::ProfileRoots;

use super::error::{error_from_envelope, CliError};
use super::target::{resolve_host, resolve_token};

/// A thin skin over one HTTP request to the daemon.
pub struct ApiClient {
    base: String,
    host: String,
    is_local: bool,
    token: Option<String>,
    client: reqwest::Client,
}

impl ApiClient {
    /// Discover the target + token per §1.5 and build the client.
    pub fn new(host: Option<String>, roots: &ProfileRoots) -> Self {
        let target = resolve_host(host);
        let token = resolve_token(&target, roots);
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");
        ApiClient {
            base: format!("http://{}", target.addr),
            host: target.addr,
            is_local: target.is_local,
            token,
            client,
        }
    }

    /// The target `ip:port` (for the daemon-down copy + the status header line).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Whether the target is the local daemon (governs the linger check).
    pub fn is_local(&self) -> bool {
        self.is_local
    }

    pub async fn get(&self, path: &str) -> Result<Value, CliError> {
        let req = self.bearer(self.client.get(format!("{}{}", self.base, path)));
        self.send(req).await
    }

    /// P0 mutation transport — consumed by the T7 transport verbs
    /// (play/pause/toggle/stop/next/prev/seek/volume/mute).
    pub async fn post(&self, path: &str, body: Value) -> Result<Value, CliError> {
        let req = self.bearer(self.client.post(format!("{}{}", self.base, path)).json(&body));
        self.send(req).await
    }

    fn bearer(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<Value, CliError> {
        let resp = req.send().await.map_err(|e| self.classify_transport(e))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if status.is_success() {
            match serde_json::from_str::<Value>(&text) {
                Ok(v) => Ok(v),
                // Unparseable 2xx body → the §1.6 sanctioned second request.
                Err(_) => Err(self.diagnose_skew().await),
            }
        } else {
            match serde_json::from_str::<Value>(&text) {
                Ok(v) => Err(error_from_envelope(&v)),
                Err(_) => Err(self.diagnose_skew().await),
            }
        }
    }

    fn classify_transport(&self, e: reqwest::Error) -> CliError {
        if e.is_connect() || e.is_timeout() {
            CliError::Unreachable(self.host.clone())
        } else {
            CliError::Runtime(format!("request failed: {e}"))
        }
    }

    /// §1.6: an unreadable body / unknown envelope is the only trigger for the
    /// single sanctioned second request, `GET /api/info` — the stable identity
    /// route. `api_version` mismatch → refuse politely; otherwise a plain error.
    async fn diagnose_skew(&self) -> CliError {
        if let Some(api) = self.info_api_version().await {
            if api != crate::API_VERSION {
                return CliError::ApiSkew {
                    daemon: api,
                    cli: crate::API_VERSION,
                };
            }
        }
        CliError::Runtime("daemon returned an unreadable response".to_string())
    }

    async fn info_api_version(&self) -> Option<u32> {
        let req = self.bearer(self.client.get(format!("{}/api/info", self.base)));
        let resp = req.send().await.ok()?;
        let v: Value = resp.json().await.ok()?;
        v.get("api_version").and_then(|a| a.as_u64()).map(|a| a as u32)
    }
}
