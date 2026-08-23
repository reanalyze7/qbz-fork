use qbz_app::settings::scrobblers::ScrobblerSettingsStore;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

pub(super) fn open_store(roots: &ProfileRoots) -> Result<ScrobblerSettingsStore, i32> {
    ScrobblerSettingsStore::new_at(&roots.data).map_err(|e| {
        eprintln!("error: cannot open the scrobbler store: {e}");
        1
    })
}

pub(super) fn not_connected(provider: &str) -> i32 {
    eprintln!("error: {provider} is not connected");
    eprintln!("  → connect it first: qbzd scrobble login {provider}");
    1
}

/// Best-effort: tell a running daemon to reload so the scrobble-on-play driver
/// picks up new credentials. Silent if the daemon is down — the store write is
/// what matters.
pub(super) async fn nudge_reload(host: Option<String>) {
    let roots = crate::paths::ProfileRoots::resolve(None, None);
    let client = ApiClient::new(host, &roots);
    let _ = client.post("/api/settings/reload", serde_json::Value::Null).await;
}
