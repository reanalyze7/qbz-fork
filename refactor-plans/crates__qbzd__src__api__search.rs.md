# crates/qbzd/src/api/search.rs (340 lines)

## 1. Summary

Implements `GET /api/search` for the qbzd control-plane daemon: the
`search()` HTTP handler (auth gate, fan-out to up to four typed Qobuz
searches, JSON envelope assembly), small internal helpers
(`auth_gate`, `upstream_error`), the `Category`/`SearchType`/
`SearchParams` types, `parse_query` (query-string parsing with clamping
defaults), and a test suite for `parse_query`/`SearchType`.

## 2. Proposed module split

| New file | Owns | ~lines |
|---|---|---|
| `api/search/mod.rs` | The exported `search()` handler + module doc; re-exports | ~90 |
| `api/search/params.rs` | `Category`, `SearchType`, `SearchParams`, `parse_query`, `DEFAULT_LIMIT`/`MAX_LIMIT` constants | ~110 |
| `api/search/errors.rs` | `auth_gate`, `upstream_error` — the two response-building helpers | ~35 |
| `api/search/tests.rs` | The entire `#[cfg(test)] mod tests` block | ~80 |

This is a small file (340 lines, ~2.6x the limit) so a light 3-way split
(handler / query-parsing types / error helpers) plus tests is enough —
no need for a deeper pure/IO breakdown given the handler itself is
already a thin, mostly-declarative fan-out.

## 3. Re-export / public API surface

`api/search/mod.rs` keeps the `search` function itself (the actual route
handler, so it makes most sense as the file other code imports from) and
re-exports/pulls in the rest:

```rust
mod errors;
mod params;
#[cfg(test)]
mod tests;

use errors::{auth_gate, upstream_error};
use params::{parse_query, Category, SearchType};

pub fn search(state: &ApiState, query: &str) -> Response<Cursor<Vec<u8>>> {
    // unchanged body
}
```

Since `Category`/`SearchType`/`SearchParams` are not used outside this
file today (grep confirms `search.rs` is self-contained), they can stay
private to `params.rs` (only `search()` in `mod.rs` needs them via a
`use` import) — no crate-level re-export needed. The router that
dispatches `GET /api/search` to `super::search::search` (per the
existing `mod.rs`/router pattern in `api/`) is unaffected since the
module path `api::search` doesn't change, only becomes a directory.

## 4. Tricky coupling/shared state to watch out for

- `SearchType::as_str()` is used both in the handler (`params.stype.as_str()`
  for the echoed `"type"` field) and directly in tests — keep it on
  `SearchType` in `params.rs`, no change needed there.
- `search()`'s per-category fan-out (`do_albums`/`do_tracks`/etc.) reads
  `params.stype` and matches on `Category` variants inline — this stays
  in `mod.rs` since it's the handler's own branching logic, not really
  "params" logic; only the parsing itself moves.
- The four `state.rt.block_on(state.runtime.core().search_*(...))` calls
  are structurally identical (same shape × 4 categories) — a tempting
  refactor-while-splitting is to factor them into one helper, but the
  instructions say move-only, no behavior change; keep the four blocks
  as-is in `mod.rs` during the split, note the duplication for a
  possible follow-up cleanup instead of doing it now.
- The extensive header comment (spec references, blacklist-not-applied
  rationale, auth-gate rationale) is important design context — split
  it: the "server-side shaping"/"blacklist not applied" parts belong on
  `mod.rs` (they describe the handler's behavior), while nothing in that
  comment specifically concerns `params.rs` or `errors.rs`, so those can
  get shorter, focused doc comments instead.

## 5. What to verify after the real split

- `cargo build -p qbzd` and `cargo test -p qbzd api::search::` — all 8
  tests green (defaults, percent-decoding, each typed category, missing/
  blank query rejection, unknown-type rejection, limit clamping/bad
  numbers, `SearchType::as_str` round-trip).
- Confirm the daemon's route table still resolves `/api/search` to this
  handler (check `crates/qbzd/src/api/mod.rs` or wherever routes are
  registered) — the function name/signature (`search(&ApiState, &str) ->
  Response<Cursor<Vec<u8>>>`) is unchanged, so this should be a non-issue,
  but confirm no macro or route table references the file path directly.
- Smoke-test via the CLI: `qbzd search "some query"` (and `--json`) for
  each `--type` value, confirming responses are byte-identical to
  pre-split behavior (same envelope shape, same clamped limit/offset
  echoing).
