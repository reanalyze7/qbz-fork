# `crates/qbz-ui/ui/shell/HeaderMenuOverlay.slint` (163 lines)

Full-shell overlay drawing the open header/sidebar nav-tab dropdown as a plain (non-
pointer-grabbing) Rectangle, so hover-switching between tabs works. Only marginally over
the 130-line limit.

## Proposed split

Given the tight hover/timer/state coupling, a light split is safer than a heavy one:

- `HeaderMenuOverlay.slint` (~100 lines) — stays the public surface: `export component
  HeaderMenuOverlay`, all the hover-tracking properties (`panel-hovered`, `hovered-row`,
  `menu-hovered`), the idle `Timer`, the scrim `TouchArea`, and the outer `panel`
  Rectangle shell.
- `shell/HeaderMenuPanel.slint` (~65 lines) — extract just the inner `panel-box`
  contents (optional title + hairline + the `for entry[idx] in HeaderMenuState.items`
  loop, lines ~99-160) into its own component, taking callbacks `row-hover-changed(idx,
  bool)` and `item-clicked(idx)` forwarded back up to the parent so `hovered-row` and
  `navigate`/`close-menu` logic stays centralized in the overlay (which owns the timer
  logic that depends on it).

## Coupling to flag

- This is a delicately-timed component (1s idle-close timer paused by hover state
  spread across panel/row/trigger) — the header comment explains WHY (PopupWindow pointer-
  grab broke hover-switching). Any split must preserve that `menu-hovered` OR-combination
  exactly; don't let `hovered-row`/`panel-hovered` state silently move out of the parent
  that owns the `Timer`.
- `changed open-index => { ... }` reset logic depends on `panel-hovered`/`hovered-row`
  living in the same component as the `Timer` — keep them together even if the panel's
  visual content is extracted.

## Verify after split

- Slint compile check.
- Manual test: open a header tab menu, hover a sibling tab (must hover-switch instantly,
  not wait for the idle timer), hover the panel/rows (idle timer must pause), move away
  (timer resumes and closes after ~1s), click outside (closes immediately), sidebar
  flyout variant (`from-sidebar`) positions correctly.
