# crates/qbz-ui/ui/settings/IntegrationsSettings.slint (311 lines)

## Summary
Settings > Integrations panel: Recommendations opt-out, MusicBrainz opt-out, and a
collapsible master "Scrobblers" section with per-service (Last.fm, ListenBrainz)
connect/disconnect rows bound to `ScrobbleState`/`ScrobbleActions`. Also defines two
small local components (`GroupHeader`, `SecondaryButton`) reused throughout.

## Proposed split
Slint requires each component be defined whole in one file, so the split extracts
the large collapsible SCROBBLERS body and its two local helper components into their
own files, composed back by the top-level component.

- `settings/IntegrationsSettings.slint` (~90 lines) — top-level `IntegrationsSettings`
  component: the RECOMMENDATIONS + METADATA rows (both short, MusicBrainz-adjacent
  opt-outs), then instantiates the extracted `ScrobblersSection` component in place
  of the current inline HorizontalLayout + `if ScrobbleState.enabled` body.
- `settings/integrations-settings/GroupHeader.slint` (~10 lines) — the shared
  `GroupHeader` local component, promoted to its own file (also used unchanged by
  `PlaybackSettings.slint` currently as a local duplicate — see coupling note).
- `settings/integrations-settings/SecondaryButton.slint` (~50 lines) — the
  `SecondaryButton` component (the shared 160px settings-row button).
- `settings/integrations-settings/ScrobblersSection.slint` (~230 lines) — a new
  `ScrobblersSection` component: the master header row (title + collapse chevron +
  master toggle) and the conditional body (Last.fm block + ListenBrainz block +
  shared status line). At ~230 lines this alone still needs splitting:
  - `settings/integrations-settings/ScrobblersSection.slint` (~90 lines) — header row
    + master toggle + collapse chevron + hosts the two service sub-components below
    + the shared status-text line.
  - `settings/integrations-settings/LastFmRow.slint` (~90 lines) — the LAST.FM group
    (enable toggle, connect/finish two-step flow, disconnect row).
  - `settings/integrations-settings/ListenBrainzRow.slint` (~90 lines) — the
    LISTENBRAINZ group (enable toggle, token `LineEdit` + save, disconnect row).

## Re-export surface
`settings/IntegrationsSettings.slint` stays the single import surface — the
`export component IntegrationsSettings` signature (callback `settings-bool`) is
unchanged, and none of the extracted sub-components are exported/imported elsewhere,
so the existing `import { IntegrationsSettings } from "settings/IntegrationsSettings.slint";`
call site (SettingsView or similar) needs no change.

## Coupling / watch out
- `GroupHeader` and `SecondaryButton` are currently duplicated verbatim in
  `PlaybackSettings.slint` (same component bodies, same doc comment referencing
  "AppearanceSettings/OfflineSettings' SecondaryButton"). If this split promotes them
  to shared files, **PlaybackSettings.slint's own split (see its plan) should import
  the SAME shared files instead of keeping its own copies** — flag this to whichever
  agent/pass actually performs the code split, since it's an opportunity to
  de-duplicate across the two files this batch covers.
- The collapse chevron `TouchArea` toggles `ScrobbleState.ui-collapsed` AND calls
  `ScrobbleActions.collapse-toggle(...)` — keep both statements together in
  `ScrobblersSection.slint`'s header block.
  the two service rows.
- `UiFocusState.text-input-focused` wiring (the `guard-focused` prop workaround for
  `std-widgets` `LineEdit`) must move intact into `ListenBrainzRow.slint` — this is a
  documented hotkey-guard probe (issue #619), don't simplify it away during the move.
- `ScrobbleActions.load()` is called in `init =>` on the TOP-LEVEL
  `IntegrationsSettings` component — keep this in the top file, not in
  `ScrobblersSection`, since it must fire once per panel mount regardless of the
  scrobblers section's enabled/collapsed state.

## Verify after split
- Slint compile check (`slint-viewer` or the project's build) on
  `IntegrationsSettings.slint` and its importer (SettingsView).
- Visual smoke-test: Recommendations/MusicBrainz toggles still persist; Scrobblers
  master toggle still enables/disables + collapses the body; Last.fm connect →
  open-auth-url → finish flow still works; ListenBrainz token paste + save +
  disconnect still work; status line color-codes correctly (error/success/muted).
