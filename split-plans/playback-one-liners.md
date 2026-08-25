# playback one-liners: `meta/artwork.rs` + `local/folder.rs`

Two files at **131 lines** — one line over the 130-line budget each. Both are
handled below with the smallest change that is honest; neither is a real
multi-responsibility file, so neither gets a real split.

Verified with `wc -l`: both are 131 lines exactly.

---

## 1. `crates/qbz/src/playback/meta/artwork.rs` (131 → 30 + ~104)

### 1. Why is this file long?

Not multi-responsibility in the usual sense, but not one thing either. It holds
two independent spawn-and-apply entry points:

| lines | item | what it does |
|---|---|---|
| 7–28 | `load_now_playing_artwork` | decodes the cover at 160px, sets `NowPlayingState.artwork` |
| 30–131 | `load_now_playing_artwork_large` | decodes at 1000px, sets `artwork-large` **and** derives the whole immersive ambient set (bg texture, glow, spectrum pair, lyrics accent, wgpu shader palette) |

The second function is 102 of the 131 lines on its own, and it grew that way for
a documented reason: all pixel crunching was deliberately pulled off the UI
thread into that one closure, with long comments explaining why. The 1-line
overage is entirely attributable to it.

**Is there dead weight?** No. Checked:
- `use slint::ComponentHandle;` — needed for `.global::<…>()`.
- `crate::{AppWindow, ImmersiveState, NowPlayingState}` — all three used.
- The multi-line signature at 36–39 is rustfmt output (the one-line form is 106
  chars, over the default `max_width = 100`; the repo has no `rustfmt.toml`, so
  the default applies). It cannot be collapsed.
- The only remaining way to win one line is deleting a blank line or a comment
  line. Those comments record real bug history (the atmosphere-flicker note,
  the "URL dedupe left bg-image empty" note). **Deleting one to win a line is
  not proposed** — that is exactly the mangling the README warns against.

So option (b) is unavailable and option (c) would mean sacrificing a comment.

### 2. What are the seams?

The clean seam is between the two functions: they share nothing but the two
early-return guards (`art.is_empty()`, `shared_cache()`), which are four lines
of boilerplate, not shared state. There is a second, finer seam inside the big
function — the pure pixel block at 65–97 (`artwork_buf` + `cover_tiny_samples`
analysis) is pure computation over `(pixels, w, h)` and could become a
pure/IO split. **That one is not recommended**: it requires inventing a carrier
type for the 5-tuple that crosses into the event loop, which is real logic churn
for a one-line problem.

### 3. Recommendation — option (a), by function, as a verbatim move

Move `load_now_playing_artwork_large` (lines **30–131**, doc comment included)
into a new sibling:

**New file:** `crates/qbz/src/playback/meta/artwork_large.rs`

Its header, mirroring the existing module style:

```rust
//! The higher-res now-playing cover decode: feeds the hover preview and the
//! immersive ambient background/glow/spectrum palette.
use slint::ComponentHandle;

use crate::{AppWindow, ImmersiveState, NowPlayingState};
```

Then lines 30–131 pasted unchanged. No body edit, no signature change — the
function already uses only `crate::`-absolute paths (`crate::artwork`,
`crate::immersive`, `crate::shader_underlay`), so nothing `super::`-relative
moves with it. Result: ~104 lines, under budget.

**In `crates/qbz/src/playback/meta/mod.rs`**, add to the `mod` block (alphabetical,
after `mod artwork;` on line 5):

```rust
mod artwork_large;
```

**In `artwork.rs`**, re-export from the original path so no importer changes:

```rust
pub(super) use super::artwork_large::load_now_playing_artwork_large;
```

`artwork.rs` ends at ~30 lines (the 160px bar decode plus that re-export).

### 4. Public surface / callers

`load_now_playing_artwork_large` has exactly one caller:
`crates/qbz/src/playback/meta/push_ui.rs:5`

```rust
use super::artwork::{load_now_playing_artwork, load_now_playing_artwork_large};
```

The re-export line above keeps `super::artwork::load_now_playing_artwork_large`
resolving, so **push_ui.rs is untouched**. Nothing else in `crates/` references
either function outside doc/prose comments (`fields.rs:81`, `push_ui.rs:90` —
both mention it by name in prose only; those names stay valid). Neither function
is re-exported from `playback/mod.rs`; both are `pub(super)`, so the blast radius
is the `meta/` module and nothing else.

If a reviewer prefers no re-export shim, the alternative is a one-line edit in
push_ui.rs splitting the import into two `use` lines. Either is fine; the shim
is chosen because README point 3 asks for importers not to change.

**Not recommended:** leaving the file at 131 with a documented exception. The
seam here is genuine (two unrelated entry points), so an exception is not
warranted — unlike an irreducible enum or declaration list.

---

## 2. `crates/qbz/src/playback/local/folder.rs` (131 → 101)

### 1. Why is this file long?

Four sibling entry points, all the same shape (spawn → load tracks → 
`play_local_tracks_now`). Cohesive, correctly grouped. The file is long by one
line because **one of the four is dead code**.

### 2. What are the seams? — the dead function

Lines **103–131**: `play_local_tracks_from`, carrying `#[allow(dead_code)]` and a
doc comment that says it outright: *"(Superseded by the instant in-memory-cache
path; kept for the full-list option.)"*

Verified by grep over `crates/` for `play_local_tracks_from`:
- `crates/qbz/src/playback/local/folder.rs:108` — its own definition.
- `crates/qbz/src/playback/local/album.rs:31` — a prose comment only.

That is the complete result set. It is also **not re-exported**:
`local/mod.rs:15–17` lists only `play_local_folder_recursive`,
`play_local_folder_tracks_from`, `play_local_tracks`; `playback/mod.rs:59–62`
mirrors that list. So the function is unreachable from outside the file and
unused inside it — the `#[allow(dead_code)]` is what has been suppressing the
compiler from saying so.

This is real dead weight, not a comment sacrificed to win a line.

### 3. Recommendation — option (b), delete

Delete lines **102–131** (the blank separator line, the 4-line doc comment, the
`#[allow(dead_code)]` attribute, and the function body). File lands at **101
lines**.

- No new module file.
- No change to `local/mod.rs` — the function is not in its `pub use` list.
- No change to `playback/mod.rs` — same.
- No caller changes anywhere: there are no callers.

Follow-up, in the same commit: `album.rs:31` says *"The sibling play paths
(play_local_album / play_local_tracks_from / the Qobuz play-all paths)"*. Update
that name to `play_local_folder_tracks_from`, which is the live sibling it
actually describes. This is a comment correctness fix, not a line-count move.

**Not recommended — option (a):** moving `play_local_tracks_from` into a sibling
module (e.g. `folder_search.rs`) would also fix the count, but it relocates dead
code and keeps the `#[allow(dead_code)]` alive, adding a file for nothing. If
the team explicitly wants the function preserved as a reference implementation,
that is the fallback shape; otherwise `git log`/`git show` is the archive.

**Not recommended — option (c):** the file *is* cohesive, so "tighten instead of
split" is tempting, but there is a genuine deletion available, which is strictly
better than shaving whitespace.

---

## Risks

**`meta/artwork.rs`**
- The re-export `pub(super) use super::artwork_large::…` inside `artwork.rs`
  must be `pub(super)`, not `pub` — `pub` on a private module's item triggers
  `unreachable_pub`-style noise and changes the effective visibility story.
  Matching the original `pub(super) fn` keeps the surface identical.
- `mod artwork_large;` must be declared in `meta/mod.rs`, not in `artwork.rs`.
  Declaring it inside `artwork.rs` would nest it as `meta::artwork::artwork_large`
  and break the `super::artwork_large` path used by the re-export.
- The moved body touches `ImmersiveState` and `NowPlayingState`; both must be
  imported in the new file or the move fails to compile. `slint::ComponentHandle`
  is required in **both** files (each calls `.global::<…>()`), so it stays in
  `artwork.rs` as well — do not "clean it up" out of the original.
- Low behavioural risk: the move is verbatim, the function is `tokio::spawn`ed
  from one call site, and thread/Send properties are unchanged.

**`local/folder.rs`**
- Deleting a `#[allow(dead_code)]` function is safe only if nothing reaches it
  reflectively. Rust has no such path here, and the grep above is exhaustive
  over `crates/`. Re-run `grep -rn play_local_tracks_from crates/` before
  deleting, in case a branch merged a caller in the meantime.
- The function is the only one distinguishing `search_with_filter` usage on this
  playback path. If the "full-list option" it was kept for is on someone's
  roadmap, deletion costs a `git show` to restore. Confirm with whoever added
  the `#[allow]` if that is cheap to do; otherwise proceed — dead code with a
  suppression attribute is the standard case for removal.
- After deletion the compiler will no longer need the `#[allow(dead_code)]`
  anywhere in the file; make sure no other attribute or import (`fill_missing_covers`,
  `play_local_tracks_now`) becomes unused. Both are still used by the three
  remaining functions, so no import should be dropped.

**Both**
- Verify with `cargo check -p qbz` and `wc -l` on the touched files after the
  edits. Expected: `artwork.rs` ~30, `artwork_large.rs` ~104, `folder.rs` 101.
