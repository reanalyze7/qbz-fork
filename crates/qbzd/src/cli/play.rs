// crates/qbzd/src/cli/play.rs — the `qbzd play [CONTENT]` verb (02 §2.3).
//
// Bare `qbzd play` resumes / cold-starts the current queue (the shipped P0
// route POST /api/playback/play, delegated to `transport::play` so there is
// one resume implementation). With a content argument it plays that content
// via POST /api/play: `track:ID` | `album:ID` | `artist:ID` | `playlist:ID` |
// a Qobuz URL | a bare numeric track id. Exit codes come from the frozen table
// via `CliError`; a malformed selector is a local usage error (exit 2).
mod play_format;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use play_format::{render, to_body};

pub async fn play(host: Option<String>, content: Option<String>, roots: &ProfileRoots) -> i32 {
    let content = match content {
        // Bare `play` = resume, the shipped behaviour (one implementation).
        None => return crate::cli::transport::play(host, roots).await,
        Some(c) => c,
    };

    let body = match to_body(&content) {
        Ok(b) => b,
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!(
                "  → try: qbzd play album:<ID> | track:<ID> | artist:<ID> | playlist:<ID> | <qobuz-url>"
            );
            return 2;
        }
    };

    let client = ApiClient::new(host, roots);
    match client.post("/api/play", body).await {
        Ok(v) => {
            println!("{}", render(&v));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}
