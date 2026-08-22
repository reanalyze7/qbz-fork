# `crates/qbz-ui/ui/mix/MixView.slint` (322 lines)

## 1. Summary
Qobuz mix detail view (DailyQ/WeeklyQ/FavQ/TopQ): a gradient-artwork header
with title/description/track-count + circular action buttons (play/shuffle/
save/multi-select/refresh), a bulk-action bar, and a `TrackRow`-based track
list with a scrollbar.

## 2. Proposed module layout

**Important finding before splitting:** this file defines a LOCAL
`component CircleAction inherits Rectangle` (lines ~20-53) that duplicates
the shared, exported `crates/qbz-ui/ui/primitives/CircleAction.slint`.
That shared primitive's own doc comment literally says its `on-surface`
variant "mirrors the theme-aware circular buttons used on plain page
backgrounds instead... (MixView pattern)" — meaning the shared component was
already generalized FROM this exact view's pattern but `MixView.slint` was
never migrated to use it. **The recommended split is not to extract
MixView's local `CircleAction` into its own file, but to delete it and have
`MixView.slint` import the shared `primitives/CircleAction.slint` with
`on-surface: true`, `primary` and `active` as needed.** This removes ~35
duplicated lines outright rather than relocating them. Flag this explicitly
for whoever does the real split — it's a correctness/consistency win, not
just a line-count exercise, but verify visually (screenshot/manual check)
since the two implementations are NOT pixel-identical (different hover
colors, different focus-ring/tooltip support in the shared version).

Remaining split, in `crates/qbz-ui/ui/mix/`:

- `mix/MixView.slint` (~130) — stays the export; keeps the outer
  `Flickable`/scroll-restore logic, the top nav-buttons row, and composes
  the header + bulk-action-bar + track-list pieces below. Imports
  `CircleAction` from `../primitives/CircleAction.slint` per the finding
  above instead of defining its own.
- `mix/MixHeader.slint` (~120) — the gradient artwork square + metadata
  column (kind label, title, description, track-count/duration line, action
  button row). Takes the mix-gradient `brush`, title/subtitle/counts as `in`
  properties and re-exposes `play-all`/`shuffle`/`media-action` callbacks
  for `MixView` to wire up (mirrors the existing callback-bubbling pattern
  already used for `play-track`/`play-all`/etc.).
- `mix/MixTrackList.slint` (~130) — the column header row, loading spinner /
  empty-state text, the `for track[i] in MixState.tracks: TrackRow` loop,
  and the `ListScrollbar` wiring (currently anchored to `flick` in the
  outer file — see coupling note below on how to keep that anchoring
  correct once it moves).

## 3. Re-export / public API surface
`crates/qbz-ui/ui/mix/MixView.slint` remains the sole import path (current
importers per `grep -rl MixView`: `AppShell.slint`,
`playlist/PlaylistView.slint`). `MixHeader.slint`/`MixTrackList.slint` are
new internal-only files imported solely from `MixView.slint`.

## 4. Tricky coupling to watch
- **The `CircleAction` name collision is the main risk here.** If the split
  is done mechanically (just copy MixView's local `CircleAction` into a new
  file) it will either (a) shadow the shared primitive if both are somehow
  imported under the same alias, or (b) perpetuate the duplication the
  shared component's own docs say it replaced. Whoever executes this split
  should compare both implementations side-by-side and either adopt the
  shared one (preferred, per the doc trail) or, if there's a subtle visual
  reason MixView's differs (e.g. simpler hover states, no tooltip), name the
  local copy something distinct like `MixCircleAction` to avoid ambiguity —
  never leave two components literally named `CircleAction` importable into
  the same scope.
- `ListScrollbar`'s `viewport-height`/`viewport-y` are bound to the outer
  `flick := Flickable`'s properties (`flick.viewport-height`, two-way
  `viewport-y <=> flick.viewport-y`) — since `ListScrollbar` is declared as
  a sibling of `flick` at the `MixView` root (not inside `MixTrackList`),
  keep the scrollbar instantiation in `MixView.slint` itself even though the
  track list content moves to `MixTrackList.slint`; only the track-list rows
  extract, not the scrollbar wiring.
- The scroll-position-restore logic (`NavState.restore-scope ==
  "mix"`/`NavState.scroll-restore`/`sr-restore()`) lives on the `flick`
  Flickable and must stay in `MixView.slint` for the same reason — it's
  bound to the specific `Flickable` instance, not to the track list content.
- `MixState.multi-select` gates both the header's "select" `CircleAction`
  AND the bulk-action bar's visibility AND `MixTrackList`'s
  `multi-select-mode` on every `TrackRow` — this one flag threads through
  all three extracted pieces; make sure each new component takes it as an
  explicit `in property` rather than assuming it can reach the global
  directly (it's a global today, `MixState.multi-select`, so direct access
  actually still works — just don't accidentally introduce a stale local
  copy).
- `root.media-action(kind, id, action)` bubbling: `MixTrackList`'s
  `TrackRow.media-action` handler intercepts `"play"` locally
  (`root.play-track(id)`) and forwards everything else via
  `root.media-action(kind, id, action)` — preserve this interception exactly
  when the loop moves into `MixTrackList.slint` (it needs its own
  `play-track` callback to bubble further up to `MixView.slint`).

## 5. What to verify after the real split
- Slint compile check / `cargo build -p qbz-ui`.
- Visual smoke-test of a mix detail view: header gradient + title render,
  play/shuffle/multi-select/refresh buttons work, bulk-action bar appears
  in multi-select mode, track list renders with scrollbar, and — critically
  — the action buttons look/behave the same (or intentionally better) after
  switching to the shared `CircleAction` primitive.
- Confirm `AppShell.slint` and `PlaylistView.slint` (the two known
  importers) still compile unchanged.
