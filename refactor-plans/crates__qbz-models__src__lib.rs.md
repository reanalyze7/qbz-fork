# crates/qbz-models/src/lib.rs (136 lines)

Crate root: module declarations + one large `pub use types::{...}` re-export
list (~90 types).

## Proposed split

- `lib.rs` (~45 lines) — keep the `pub mod` declarations, the doc comment,
  and the smaller re-exports (`error`, `events`, `lenient`, `playback`,
  `source`, `traits`).
- `types/reexport.rs` or simply reformat: since `pub use types::{...}` is
  one big list already inside `types` re-export, the simplest fix is to
  move that one `pub use types::{ ... };` block into `types/mod.rs` itself
  as `pub use self::{...}` — i.e. have `types` re-export its own public
  surface, and have `lib.rs` do a short `pub use types::*;` or
  `pub use types::prelude::*;`. This removes the ~90-line list from
  `lib.rs` entirely without adding a new file.

## Tricky coupling

- `qbz_models::{Track, Album, ...}` is imported crate-root-style across
  the whole workspace (`use qbz_models::Track` etc.) — the fix above keeps
  those import paths unchanged since `pub use types::*` (or an equivalent
  named list from `types/mod.rs`) still surfaces every type at the crate
  root.
- Check `types/mod.rs` doesn't already have conflicting names before doing
  a glob `pub use types::*` (a named re-export list, moved as-is into
  `types/mod.rs`, is the safer option).

## Verify after split

`cargo build --workspace` (many crates depend on qbz-models re-exports —
this is the one to build widest), `cargo test -p qbz-models`.
