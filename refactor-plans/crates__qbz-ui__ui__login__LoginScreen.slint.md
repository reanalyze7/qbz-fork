# crates/qbz-ui/ui/login/LoginScreen.slint (294 lines)

## Summary
Single `LoginScreen` component: the centered login card (brand/wordmark,
ToS checkbox, sign-in button, phase-based status text, offline/init-error
callouts, "Start offline" link, legal disclaimer). No domain logic — pure
layout driven by `LoginState`/`OfflineState` globals plus 4 callbacks.

## Proposed split
By visual section, since it's one long `VerticalLayout` of independent
blocks:
- `LoginScreen.slint` (~60 lines) — **stays the public re-export/root**.
  Card chrome (`Rectangle` + outer `VerticalLayout`/`HorizontalLayout`
  centering, lines 29-46) plus composes the sections below.
- `LoginBrand.slint` (~35 lines, new) — logo image + wordmark + subtitle
  text (lines 47-73).
- `LoginTosAndAction.slint` (~55 lines, new) — ToS checkbox row + sign-in
  button (lines 77-120); owns `tos-accepted` as an in-out property passed
  from `root`.
- `LoginStatusPanel.slint` (~90 lines, new) — the phase 1/2 status texts +
  the sign-in-error box (lines 124-188); reads `LoginState` directly.
- `LoginOfflineCallouts.slint` (~90 lines, new) — the offline-connectivity
  callout, init-error callout, and "Start offline" link (lines 190-272);
  reads `OfflineState` directly, calls `root.start-offline()` via a
  forwarded callback.
- Legal disclaimer (lines 276-289, ~15 lines) can stay inline in
  `LoginScreen.slint` or fold into `LoginBrand.slint`/its own tiny file —
  implementer's call, it's static text with no logic.

## Re-export surface
`LoginScreen.slint` keeps exporting `LoginScreen` with the same
`tos-accepted` in-out property and the same 4 callbacks
(`sign-in-via-browser`, `cancel-login`, `start-offline`, `open-tos`) — the
Rust side (`src/commands.rs` per the file's own header comment) binds to
these names.

## Coupling / watch out
- `tos-accepted` is two-way bound (`checked <=> root.tos-accepted`) into
  the checkbox and read by the sign-in button's `enabled` gate — if the
  checkbox row moves to its own file, thread `tos-accepted` as an in-out
  property, don't duplicate state.
- `LoginState`/`OfflineState` are ambient globals (imported from
  `../state.slint`) — each new file needs its own `import` line, not
  inherited from the root file.
- The offline callout and init-error callout are mutually exclusive
  (`connectivity != 2` guard) — keep both `if` conditions together in one
  file so the exclusivity is easy to audit at a glance.

## Verify after split
- `cargo build -p qbz-ui`.
- Manually exercise: fresh sign-in (browser flow phases 0→1→2), a forced
  sign-in error, offline-boot with a previous session, and the "Start
  offline" guest-profile path (#553).
