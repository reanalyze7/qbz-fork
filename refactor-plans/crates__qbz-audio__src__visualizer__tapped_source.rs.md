# crates/qbz-audio/src/visualizer/tapped_source.rs (140 lines)

## 1. Summary

`TappedSource<S>` — a transparent rodio `Source` wrapper that
intercepts f32 samples and pushes them into a `RingBuffer` for
visualization (gated by an `AtomicBool` enabled flag) without altering
playback; implements `Iterator` and `Source` by delegating to the
wrapped source, plus two tests.

## 2. Proposed module split

At 140 lines (barely over the 130 limit, ~1.08x), this file is nearly
compliant already. The cleanest fix that respects the pure/IO-adjacent
split principle without over-fragmenting a tiny, cohesive wrapper type:

| New file | Owns | ~lines |
|---|---|---|
| `tapped_source/mod.rs` | `TappedSource<S>` struct, `impl TappedSource` (`new`), `impl Iterator`, `impl Source` — the whole non-test implementation, unchanged | ~90 |
| `tapped_source/tests.rs` | The entire `#[cfg(test)] mod tests` block (`test_tapped_source_passes_through`, `test_tapped_source_fills_ring_buffer`) | ~50 |

Given how small and cohesive the implementation itself is, splitting the
`Iterator`/`Source` impls apart from the struct definition would harm
readability for no real benefit (the file's job is exactly "one small
wrapper type + its two trait impls"); pulling only the tests out is
enough to bring `mod.rs` itself under 130 lines while keeping the type's
implementation whole in one place.

## 3. Re-export / public API surface

`tapped_source/mod.rs` keeps `TappedSource` itself (it's the only public
item) and just declares the test submodule:

```rust
#[cfg(test)]
mod tests;

use rodio::Source;
use std::sync::Arc;
use std::time::Duration;

use super::RingBuffer;

pub struct TappedSource<S>
where
    S: Source<Item = f32>,
{
    // unchanged
}
// ... impls unchanged
```

Every caller doing `use crate::visualizer::tapped_source::TappedSource;`
(or however `visualizer/mod.rs` re-exports it) keeps working unchanged —
the module path `visualizer::tapped_source` still resolves to a file at
the same logical location (now a directory with `mod.rs`), and
`TappedSource` itself doesn't move.

## 4. Tricky coupling/shared state to watch out for

- `tests.rs` needs `use super::TappedSource;` plus its own imports
  (`rodio::buffer::SamplesBuffer`, `std::num::NonZero`,
  `std::sync::atomic::AtomicBool`) — currently these come via `use
  super::*;`; after the split, either keep `use super::*;` (works fine,
  `TappedSource` and the `rodio`/`std` imports used only inside the impl
  blocks aren't needed by tests anyway) or list them explicitly — either
  is fine here since it's a tiny, low-risk file.
- `super::RingBuffer` — confirm `RingBuffer` is declared in the parent
  `visualizer` module (or a sibling file) and that this relative import
  still resolves once `tapped_source.rs` becomes `tapped_source/mod.rs`
  (it does; `super::` from a `mod.rs` one level deeper still points to
  the same parent module as before, since `mod.rs` files don't add a
  nesting level relative to `super`).
- This is one of the smallest files in the batch — worth flagging to
  other agents that a 140-line file barely over the limit may not be
  worth a deep split; a single "pull tests into their own file" move is
  proportionate and this plan intentionally does the minimum.

## 5. What to verify after the real split

- `cargo build -p qbz-audio` and
  `cargo test -p qbz-audio visualizer::tapped_source::` — both tests
  green (`test_tapped_source_passes_through`,
  `test_tapped_source_fills_ring_buffer`).
- Grep the workspace for `TappedSource` usages (the audio playback
  pipeline that wraps its rodio source for the visualizer feature) to
  confirm the import path is unaffected by the file-to-directory
  conversion.
- Smoke-test: play a track with the visualizer enabled, confirm the
  visualization still animates from live audio and playback audio is
  bit-identical/unaffected (the tap must remain fully transparent).
