// crates/qbzd/src/login/ — `qbzd login` / `qbzd logout` (02-cli-and-api.md §2.2,
// memo D6). Ported from the desktop system-browser OAuth (crates/qbz/src/auth.rs)
// with three deliberate daemon changes:
//
//   1. The one-shot listener binds an EPHEMERAL port (`bind((host, 0))`), NEVER
//      the control-API port (D6 — this is what dissolves the loopback-vs-LAN 401
//      contradiction: the callback lives on its own throwaway listener).
//   2. A CSPRNG nonce is bound into the redirect PATH
//      (`redirect_url=http://<host>:<port>/<nonce>`) and validated against the
//      callback's request path; a mismatched or second callback is dropped. The
//      nonce lives in the PATH — not an OAuth `state` param — because the
//      working desktop flow sends no `state` and there is no evidence Qobuz
//      echoes one; the redirect URL itself is preserved verbatim.
//   3. The command only VALIDATES (live `login_with_token` / `login_with_oauth_code`)
//      and PERSISTS the token into the daemon root, then best-effort nudges a
//      running daemon to reload. It never activates a session in-process — the
//      daemon (a separate process) owns session activation.
//
// There is NO email+password surface anywhere (D6/D12): the only ways in are the
// browser flow, a pasted redirect URL, and a directly-injected token.
mod entry;
mod error;
mod io;
mod parsing;

pub use entry::{
    login_browser, login_paste, login_with_token_arg, logout, nudge_reload, nudge_reload_outcome,
    validate_token, NudgeOutcome,
};
pub use error::LoginError;
pub(crate) use io::nudge_host;
