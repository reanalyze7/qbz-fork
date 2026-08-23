// crates/qbzd/src/api/browse/ — the catalog READ verbs (02 §2.3):
// GET /api/album, /api/artist, /api/similar, /api/suggest.
//
// All auth-gated (they call the Qobuz client → NotInitialized without a
// session), all return the core's typed serde shapes verbatim (a stable
// `--json` contract, §3.1.4 — no raw catalog_search leakage), all blacklist
// fail-open by design (the daemon opens no blacklist store; a documented
// GUI-parity delta). None mutate playback — they are pure reads that feed the
// composition pipeline (`--ids` on the CLI side pipes into `queue add -`).
mod album;
mod artist;
mod errors;
mod query;
mod similar_suggest;
#[cfg(test)]
mod tests;

pub use album::album;
pub use artist::artist;
pub use similar_suggest::{similar, suggest};

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 100;
/// Seed cap when `suggest` falls back to the current queue.
const QUEUE_SEED_CAP: usize = 20;
