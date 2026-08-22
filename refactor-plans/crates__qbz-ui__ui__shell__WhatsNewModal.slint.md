# crates/qbz-ui/ui/shell/WhatsNewModal.slint (253 lines)

## Summary
The "What's New" release-notes modal (native port of Tauri's
`WhatsNewModal.svelte`): opened from the header hamburger menu, fetches the
matching GitHub release, renders its markdown body into a flat block model
(`crate::whats_new::render_markdown`) into `WhatsNewState`, and displays a
TOC of level-0 section headings plus the body blocks (sections/bullets/
paragraphs/links). Single component today — no sub-components exist yet, so
the split is about extracting inline repeated blocks into their own files.

## Proposed split
By repeated inline block — new small components under a `shell/whats_new/`
sibling directory, composed by a slimmed `WhatsNewModal.slint`.

- `shell/whats_new/toc_chip.slint` (~35 lines) — new component `WnTocChip`
  extracted from the `for entry in WhatsNewState.toc: ...` body (lines
  131-149): the bordered index chip showing `entry.label`.
- `shell/whats_new/body_block.slint` (~95 lines) — new component
  `WnBodyBlock` extracted from the `for block in WhatsNewState.blocks: ...`
  body (lines 164-219): renders one of section (kind 0) / bullet (kind 1) /
  paragraph (kind 2) / whole-line link (kind 3), forwarding link clicks via
  a `callback open-url(string)`.
- `shell/WhatsNewModal.slint` (~135 lines) — the slimmed main export: scrim +
  card + header (title/date + close-x) + the body `Flickable` that loops
  `for entry in WhatsNewState.toc: WnTocChip { ... }` and
  `for block in WhatsNewState.blocks: WnBodyBlock { ...; open-url(url) => { WhatsNewActions.open-url(url); } }`
  + footer Close button.

## Re-export surface
`shell/WhatsNewModal.slint` stays the file other `.slint` imports
(`import { WhatsNewModal } from "../shell/WhatsNewModal.slint"`) — unchanged
export name.

## Coupling / watch out
- `WnBodyBlock`'s `block.kind`/`block.level` semantics (0 section / 1 bullet
  / 2 paragraph / 3 link; `level` controls bullet indentation and glyph:
  `•` vs `◦` for level >= 2) are the trickiest part of this file — carry the
  exact conditional expressions over verbatim, including the
  padding-top/bottom asymmetry for section vs. other blocks (16px/6px vs
  0px/7px) and the padding-left indent (`block.level * 16px` only for
  bullets).
- The link block (kind 3) needs its own `callback open-url(string)` on
  `WnBodyBlock` since `WhatsNewActions.open-url(block.url)` currently fires
  directly inline — after extraction, the main modal wires
  `open-url(url) => { WhatsNewActions.open-url(url); }` per instance.
- `WhatsNewState`/`WhatsNewActions` stay imported only in the main
  `WhatsNewModal.slint` (the TOC and block loops live there); `WnTocChip`
  and `WnBodyBlock` should take their data via `in property` (the toc entry
  / the block struct) rather than reading the globals themselves, since
  they're pure presentational rows with no independent state.
- The TOC section is itself gated on `WhatsNewState.toc.length > 0` and
  includes a trailing divider (`Rectangle` height 8px + 1px border line +
  8px) — keep that divider logic in the main modal file since it's a
  section-level layout decision, not per-chip.

## Verify after split
- Slint compile check for the crate.
- Manual smoke test: open What's New from the header hamburger menu, confirm
  loading state, empty state ("Release notes are not available"), TOC chips
  render for a release with headings, body blocks render sections/bullets
  (both indent levels)/paragraphs/links correctly, clicking a link opens the
  URL, and Close (X and footer button) both close the modal.
- Grep for `WhatsNewModal` usage to confirm its import path is unaffected.
