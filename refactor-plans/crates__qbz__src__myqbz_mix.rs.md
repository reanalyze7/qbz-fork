# crates/qbz/src/myqbz_mix.rs (245 lines)

## Summary
Rust side of the My QBZ "Random queue" DJ-mix modal: builds the discrete
slider size options from a collection's unique track count, and drives the
open (resolve + count) / shuffle (resolve + RNG-confined dedup+sample +
replace-play) async flows against the Slint `MyQbzMixState` global.

## Proposed split
By responsibility: pure size-option math, modal open/close, the
confirm/shuffle flow (the file has no tests today — none to relocate).

- `myqbz_mix/mod.rs` (~30 lines) — module doc, `pub mod` declarations,
  `pub use` re-exports of `build_size_options`, `apply_index`, `open`,
  `close`, `shuffle` so `crate::myqbz_mix::X` paths are unchanged.
- `myqbz_mix/options.rs` (~75 lines) — `Runtime` type alias,
  `SMALL_THRESHOLD`, `STEP`, `build_size_options` (pure, easily unit-
  testable — currently untested; consider adding a small test module here
  during the split per the project's "tests at each change" rule),
  `apply_options`, `apply_index` (the two functions that push computed
  options/selection into `MyQbzMixState`).
- `myqbz_mix/open_close.rs` (~90 lines) — `open`, `close_with_error`,
  `close`.
- `myqbz_mix/shuffle.rs` (~90 lines) — `shuffle` (the confirm/sample/
  replace-play flow, including the RNG-confined sync block).

## Re-export surface
`myqbz_mix/mod.rs` re-exports `build_size_options`, `apply_index`, `open`,
`close`, `shuffle` at `crate::myqbz_mix::*` — the Slint callback wiring for
the `MyQbzMixModal` component (search `crates/qbz/src/` for
`myqbz_mix::open`, `myqbz_mix::shuffle`, `myqbz_mix::apply_index`,
`myqbz_mix::close` call sites, likely in the app's main window-setup /
callback-binding module) must keep resolving at these exact paths.

## Coupling / watch out
- `build_size_options` is pure and load-bearing for a documented
  invariant ("the LAST entry is ALWAYS the All(N) option... index == len-1
  ⇒ the All entry") relied on by `apply_index`'s `idx == len - 1` check —
  keep both together conceptually even though they land in the same
  `options.rs` file; don't let a future edit move `apply_index` without
  `build_size_options`.
- The RNG confinement in `shuffle` is explicitly load-bearing per the
  module doc: `rand::rng()` (a `!Send` `ThreadRng`) is created, used, and
  DROPPED inside a synchronous `{ ... }` block that ends before the next
  `.await` (`play_all_tracks`) — when moving `shuffle` into its own file,
  copy this block verbatim and preserve the ordering comment; splitting
  the sync block itself (e.g. extracting `dedup_by_similarity`+`hybrid_
  sample` into a helper called across an await point) would break the
  `Send` future requirement and fail to compile.
- `open` and `shuffle` both call `crate::myqbz_play::load_collection` and
  `crate::myqbz_play::resolve_collection` — a real dependency on the
  `myqbz_play` module (not something to inline here), and `shuffle` also
  calls `crate::myqbz_play::play_all_tracks` — if another agent is
  splitting `myqbz_play.rs`, flag that its public fn names/signatures are
  depended on by this file.
- `close_with_error` and `close` are used by both `open`'s failure path
  and (indirectly via the `MyQbzMixState` fields) implicitly expected by
  `shuffle`'s pre-playback close — keep both in `open_close.rs` since
  `shuffle.rs` will need to `use super::open_close::close;`.

## Verify after split
- `cargo check -p qbz` — no automated tests exist for this file today;
  compiling clean is the primary automated check. Per the "tests at each
  change" rule, consider adding a unit test for `build_size_options`
  (covering `<=0`, `<50`, exact-multiple-of-50, and non-multiple cases —
  the doc comment already specifies the exact expected behavior for each)
  in `options.rs` as part of doing the real split.
- Manual smoke-test: open the DJ-mix modal on a large collection (confirm
  the slider shows 50/100/150/…/All(N) options), on a small (<50 track)
  collection (confirm only "All (N)" appears), drag the slider and hit
  Shuffle (confirm the queue replaces and playback starts at track 0),
  and on a collection whose per-album cap shrinks the sampled set, confirm
  the "Playing N of M" toast appears.
