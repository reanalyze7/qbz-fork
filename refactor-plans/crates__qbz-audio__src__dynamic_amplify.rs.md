# crates/qbz-audio/src/dynamic_amplify.rs (137 lines)

## Summary
A `rodio::Source` wrapper (`DynamicAmplify<S>`) that reads a target gain
from a shared `Arc<AtomicU32>` (f32 bit-cast) and applies it per-sample
with a 50ms linear ramp to avoid audible clicks when the gain changes —
used for real-time volume normalization.

## Proposed split
This file is only marginally over budget (137 vs 130 — 7 lines) and is a
single cohesive generic struct with three trait impls (`Iterator`,
`Source`) plus its own constructor/gain-poll method; splitting it into
multiple files would fragment one small, tightly-coupled unit for little
benefit. Two reasonable options, in preference order:

- Option A (recommended) — trim in place: the module-doc comment (lines
  1-8) and per-field doc comments are already terse; check whether a
  handful of the `#[inline]` attributes' surrounding blank lines can be
  tightened, or whether `Ordering::Relaxed` usage notes can move to a
  single doc line — if the file can drop under 130 lines through comment
  consolidation alone (it is only 7 lines over), this avoids an
  unnecessary module split for a single-struct file with no natural
  seam.
- Option B (if Option A isn't enough or the project prefers structural
  splits over trimming) —
  - `dynamic_amplify/mod.rs` (~50 lines) — module doc, `pub use
    DynamicAmplify` re-export, the struct definition + `new` +
    `poll_gain` (the `impl<S> DynamicAmplify<S>` block).
  - `dynamic_amplify/source_impl.rs` (~90 lines) — the `impl<S> Iterator
    for DynamicAmplify<S>` block (`next`, `size_hint`) and the `impl<S>
    Source for DynamicAmplify<S>` block (`current_span_len`, `channels`,
    `sample_rate`, `total_duration`).

## Re-export surface
If split, `dynamic_amplify/mod.rs` re-exports `DynamicAmplify` at
`crate::dynamic_amplify::DynamicAmplify` (i.e.
`qbz_audio::dynamic_amplify::DynamicAmplify`) — consumed by the audio
pipeline wherever normalization is wired into the rodio source chain
(search `crates/qbz-audio/src/` and the playback engine crate for
`DynamicAmplify::new` call sites before finalizing).

## Coupling / watch out
- `DynamicAmplify<S>` is generic over `S: Source<Item = f32>` — both
  `impl` blocks (in Option B) repeat this same bound; keep the bound
  identical across files (a mismatched bound would fail to compile, not
  silently diverge, so this is low-risk but worth double-checking).
- The ramp math (`ramp_step`, `ramp_remaining`, `ramp_samples`) is
  entirely private state read/written only inside `poll_gain` (in
  `mod.rs`) and `next` (in `source_impl.rs` under Option B) — since both
  live under the same parent module (`dynamic_amplify/`), private field
  access across the two files works without visibility changes, same as
  any other same-crate multi-file struct split.
- `gain_atomic: Arc<AtomicU32>` is the one piece of state shared with the
  OUTSIDE world (whatever computes and stores the target gain writes to
  the same `Arc` this struct reads from) — this is unaffected by an
  in-crate file split; just don't change the atomic's ordering semantics
  (`Ordering::Relaxed`) during the split, it's intentionally relaxed since
  only a single f32 value is being polled, not synchronizing other
  memory.
- No `#[cfg(test)]` block exists in this file today. Given it's borderline
  (7 lines over) and self-contained pure logic (`poll_gain`'s ramp-start
  decision, `next`'s per-sample ramp arithmetic), this is a good candidate
  to add a couple of focused unit tests (e.g. "gain of 0.0 leaves current_
  gain unchanged", "a real gain change starts a ramp of the expected
  step size") as part of satisfying the project's "tests at each change"
  rule — likely the actual reason to do a real file split (to make room
  for `#[cfg(test)] mod tests` without pushing further over budget) rather
  than the production code needing it.

## Verify after split
- `cargo test -p qbz-audio dynamic_amplify::` once tests are added.
- `cargo check -p qbz-audio` (and the playback engine crate consuming
  `DynamicAmplify`) to confirm the type/constructor path is unchanged.
- Manual/audio smoke-test: play a track with normalization enabled and
  confirm no audible clicks/pops at gain-change boundaries (e.g. track
  transitions with differing loudness) — this is the one file in this
  batch where a REAL AUDIO listen-through, not just a compile check, is
  the meaningful verification.
