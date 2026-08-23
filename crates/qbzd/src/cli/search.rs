// crates/qbzd/src/cli/search.rs — the `qbzd search` verb (02-cli-and-api.md
// §2.3). A stateless renderer over one `GET /api/search` (§1.1): one verb, one
// request. Three output modes — human top-hits table (default), `--ids` (ids
// one-per-line, the composition currency for `... | qbzd queue add -`), and
// `--json` (the raw api_version-stamped payload). Exit codes come from the
// frozen table via `CliError` (§1.3): 0 · 3 unreachable · 4 needs_auth · 1 else.
mod render;
#[cfg(test)]
mod tests;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use render::{collect_ids, render as render_table};

/// `qbzd search <QUERY> [--type all|albums|tracks|artists|playlists]
/// [--limit N] [--offset N] [--ids] [--json]`.
#[allow(clippy::too_many_arguments)]
pub async fn search(
    host: Option<String>,
    query: String,
    stype: String,
    limit: u32,
    offset: u32,
    ids: bool,
    json: bool,
    roots: &ProfileRoots,
) -> i32 {
    let client = ApiClient::new(host, roots);
    let path = format!(
        "/api/search?q={}&type={}&limit={}&offset={}",
        urlencoding::encode(&query),
        urlencoding::encode(&stype),
        limit,
        offset,
    );
    let payload = match client.get(&path).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            return e.exit_code();
        }
    };

    if json {
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else if ids {
        for id in collect_ids(&payload) {
            println!("{id}");
        }
    } else {
        print!("{}", render_table(&payload));
    }
    0
}
