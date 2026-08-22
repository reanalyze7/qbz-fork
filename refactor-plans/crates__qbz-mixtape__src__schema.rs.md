# crates/qbz-mixtape/src/schema.rs (135 lines)

Just over budget — SQLite migrations for mixtape_collections +
mixtape_collection_items, plus one additive ALTER on session_queue_state.

## Proposed split

Barely over 130 (135 lines, ~46 of which are the one `run_mixtape_migrations`
fn and ~46 are tests). Recommend a light split rather than fragmenting a
single migration function:

- `schema/mod.rs` (~90 lines) — re-export surface + `run_mixtape_migrations`
  (unchanged).
- `schema/tests.rs` (~45 lines) — move the `#[cfg(test)] mod tests` block
  out, included via `#[path = "tests.rs"] mod tests;`.

This alone drops `mod.rs` well under 130 with zero functional risk.

## Tricky coupling

- None — a single idempotent migration fn, no shared state across files.

## Verify after split

`cargo build -p qbz-mixtape`, `cargo test -p qbz-mixtape schema::` (3
existing migration tests: idempotency, additive column, missing-table
tolerance).
