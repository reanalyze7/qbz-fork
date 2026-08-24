// crates/qbzd/src/cli/client/ — the stateless HTTP client behind every
// networked CLI verb (02-cli-and-api.md §1.1). One verb = one request; the
// client holds no daemon state beyond target discovery (§1.5).
//
// Target discovery precedence (§1.5): `--host` > `QBZD_HOST` > local default
// `127.0.0.1:8182`. The Bearer is sent ONLY in opt-in mode: when `QBZD_TOKEN`
// is set (remote or local) or, when targeting the LOCAL daemon, the local
// `qbzd.toml` carries `[server] token`. Default: no token anywhere.
mod api_client;
mod error;
mod target;

pub use api_client::ApiClient;
pub use error::CliError;
pub use target::resolve_host;

pub(crate) use target::resolve_token;
