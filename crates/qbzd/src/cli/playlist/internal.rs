use std::io::Read;

use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

pub(super) async fn post<F: Fn(&Value) -> String>(host: Option<String>, roots: &ProfileRoots, path: &str, body: Value, render_ok: F) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.post(path, body).await {
        Ok(v) => {
            println!("{}", render_ok(&v));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// Track ids from positional args; a single `-` reads them from stdin
/// (whitespace-separated). Every id must be numeric.
pub(super) fn resolve_ids(args: Vec<String>) -> Result<Vec<u64>, String> {
    let raw: Vec<String> = if args.len() == 1 && args[0] == "-" {
        let mut buf = String::new();
        let _ = std::io::stdin().read_to_string(&mut buf);
        buf.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        args
    };
    if raw.is_empty() {
        return Err("no track ids given".into());
    }
    let mut ids = Vec::with_capacity(raw.len());
    for s in raw {
        let n: u64 = s.parse().map_err(|_| format!("'{s}' is not a numeric track id"))?;
        ids.push(n);
    }
    Ok(ids)
}
