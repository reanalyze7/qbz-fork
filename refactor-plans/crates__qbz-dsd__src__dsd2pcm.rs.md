# crates/qbz-dsd/src/dsd2pcm.rs (168 lines)

Ported dsd2pcm decimator: precomputed FIR tables + streaming translate().

## Proposed split

- `dsd2pcm/mod.rs` (~95 lines) — re-export surface, `HTAPS` const,
  `Tables` struct + `tables()`, `bit_reverse`, `Dsd2Pcm` struct +
  `new`/`translate`/`Default`.
- `dsd2pcm/tests.rs` (~40 lines) — move the `#[cfg(test)] mod tests` out,
  included via `#[path = "tests.rs"] mod tests;`.

This alone puts `mod.rs` at ~95 lines, comfortably under budget — no
functional split needed since the table-building + translate logic is one
tightly-coupled algorithm (splitting it further would hurt readability more
than it helps).

## Tricky coupling

- `bit_reverse` is `pub(crate)` and used by `crates/qbz-dsd/src/native.rs`
  (`crate::dsd2pcm::bit_reverse`) — keep that visibility/path unchanged.

## Verify after split

`cargo build -p qbz-dsd`, `cargo test -p qbz-dsd dsd2pcm::`.
