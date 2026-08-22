# crates/qbz/src/musician.rs (203 lines)

## Summary
`MusicianPageView` controller: resolves a musician (name+role) via
MusicBrainz, loads paginated "Appears On" appearances, and applies/appends
the results plus artwork jobs into `MusicianState`.

## Proposed split
Only ~73 lines over budget — a light two-way split by responsibility
(async load vs. state-application) is enough; no need for a full
subdirectory.

- `musician.rs` (~95 lines, stays at this path) — module doc, `PAGE_SIZE`
  const, `MusicianData`/`AppearanceData` structs, `load_musician`,
  `load_more_appearances` (the async MusicBrainz-facing load functions),
  `pub use musician_state::*;` (or just `mod musician_state;` +
  `pub use` of its functions) re-export.
- `musician_state.rs` (~110 lines, new sibling file next to `musician.rs`,
  declared via `mod musician_state;` in `musician.rs` or in `lib.rs`
  alongside the existing `mod musician;` declaration — whichever pattern the
  crate already uses for sibling modules) — `apply_musician`,
  `append_appearances`, `reset_musician`, `artwork_jobs`, `confidence_label`
  — the Slint-state-application + artwork-job-building half, which doesn't
  touch the network at all (pure `AppWindow`/`MusicianState` mutation plus
  one pure mapping function).

## Re-export surface
Callers currently do `crate::musician::{load_musician, load_more_appearances,
apply_musician, append_appearances, reset_musician, artwork_jobs}` (or
similar) — since `musician.rs` keeps the `mod musician_state;` declaration
and re-exports its public functions via `pub use musician_state::*;`, every
existing `crate::musician::<fn>` call site keeps compiling unchanged. If the
crate's convention is instead to declare sibling modules in `lib.rs`
(`mod musician; mod musician_state;`), then `musician_state`'s functions
would need explicit re-export via `pub use crate::musician_state::*;` in
`musician.rs`, or callers would need updating to
`crate::musician_state::apply_musician` — confirm which convention
`crates/qbz/src/lib.rs` already uses for `mod musician;` before deciding.

## Coupling / watch out
- `AppearanceData` (in `musician.rs`) is consumed by `apply_musician`,
  `append_appearances`, and `artwork_jobs` (proposed for
  `musician_state.rs`) — it needs to stay `pub(crate)` (already effectively
  is, via crate-visible struct) so `musician_state.rs` can see it; no field
  privacy issue since all fields are already `pub`.
- `confidence_label` is a small private free function only used by
  `apply_musician` — keep it in `musician_state.rs`, not `musician.rs`,
  since it's purely a state-application detail.
- No shared mutable state between the two proposed files beyond passing
  `MusicianData`/`Vec<AppearanceData>` by value — clean boundary.

## Verify after split
- `cargo check -p qbz` (no unit tests exist in this file today).
- Grep `crate::musician::` (and `crate::musician_state::` if that path
  changes) across `crates/qbz/src/` for the MusicianPageView controller
  wiring to confirm every call site still resolves.
- Smoke-test: open a musician/contributor page, verify appearances load,
  scroll to trigger "load more", and confirm artwork fills in progressively.
