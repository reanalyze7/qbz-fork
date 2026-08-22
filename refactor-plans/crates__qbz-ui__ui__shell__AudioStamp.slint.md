# crates/qbz-ui/ui/shell/AudioStamp.slint (135 lines)

## Summary
The inline 2-row now-playing quality stamp (Row 1: tier pill + detail text,
brighter; Row 2: backend/mode LEDs, dimmer) shown in both PlayerBarSmall
and the Large bar's centre cluster — includes the delivered-vs-catalog-max
downgrade logic and the gold Hi-Res badge color.

## Proposed split
This file is only 5 lines over budget (135 vs. 130) — it is a single
cohesive component with no natural sub-component boundary (there is only
one visual row implemented here; Row 2's LEDs are described in the header
comment but are NOT actually present in this file's body, meaning the
"Row 2" mentioned in the comment lives in the PARENT — PlayerBarSmall.slint
— not here). Given the task brief's explicit instruction to keep this
file's Hi-Res gold-badge logic intact as a cohesive block (recently
touched this session), the recommended split is minimal and mechanical:
extract only the computed-property block, leaving the render tree whole.

- `AudioStamp.slint` (~95 lines) — module doc, imports, `export component
  AudioStamp`: all the property declarations move to a small helper
  component OR (simpler, recommended) just trim the header comment by
  ~10-15 lines (it is verbose prose, lines 1-16, that could be shortened
  without losing the WHY) to land under 130 without splitting the logic at
  all. This is the safest option given the small overage.
- If a structural split is still wanted: `shell/audio_stamp_tooltip.slint`
  (~40 lines) — nothing to extract here in practice, since
  `quality-tooltip`/`cause-line` are simple string-computed properties
  with no visual tree of their own; Slint properties cannot live in a
  separate file independent of the component that owns them without
  becoming a whole separate component with its own `in`/`out` properties,
  which would be over-engineering for 5 lines of overage.

## Re-export surface
`AudioStamp.slint` remains the single export both `PlayerBarSmall.slint`
and the Large bar's centre cluster import from
(`import { AudioStamp } from "../shell/AudioStamp.slint";`) — unaffected
either way, since no structural split is actually recommended.

## Coupling / watch out
- **This file was recently touched this session** (Hi-Res filter toggle
  context note in the task brief, plus the gold-badge color logic
  `badge-bg`/`badge-text`/`shown-tier-is-hires` at lines 45-49) — the
  gold-badge decision is a single cohesive computed-property chain
  (`show-delivered` → `shown-tier-is-hires` → `badge-bg`/`badge-text`) that
  must NOT be split across files; recommend leaving this entire file as-is
  or only trimming comment verbosity, not restructuring the property
  chain.
- `NowPlayingState` (Slint global) is read extensively
  (`quality-tier`, `quality-downgraded`, `quality-true-detail`,
  `quality-effective-tier`, `quality-detail`, `quality-limit-cause`) — all
  within one component, no cross-file coupling.
- `TooltipState` (Slint global) is written by the `quality-hover :=
  TouchArea` handler — same file, no coupling risk.
- Given this file is used by BOTH `PlayerBarSmall` and the Large bar
  (per the header comment "so BOTH the Small bar and the Large bar ...
  can mount the same component"), any future change here has two call
  sites to verify, not just one.

## Verify after split
- `cargo build` through the Slint build step to confirm compilation.
- Smoke-test: play a Hi-Res track and confirm the gold pill badge renders
  correctly in both PlayerBarSmall and the Large bar's centre cluster;
  play a quality-downgraded stream (e.g. via a streaming-quality cap
  setting) and confirm the ↓ arrow + delivered-tier fallback + tooltip
  "Source:"/"Output:" lines still work exactly as before.
- Given the small (5-line) overage, consider whether a comment trim alone
  suffices rather than forcing an artificial multi-file split that could
  fragment the gold-badge decision chain the task brief asked to protect.
