# crates/qbz-ui/ui/settings/PlaybackSettings.slint (213 lines)

## Summary
Playback settings panel: PLAYBACK group (continue-playback, context icon, gapless,
crossfade slider), SESSION group (restore session, resume position), and STREAMING
group (stream-uncached, buffer-size slider, streaming-only, retry-behavior select).
Backed by `SettingsState` and persisted through Rust callbacks
(`settings-bool`/`settings-select`/`settings-slider`/`settings-string`). Defines two
local components (`GroupHeader`, `Divider`) duplicated from other settings panels.

## Proposed split
Only ~83 lines over budget, and the file is naturally three group sections — extract
each group's body into a small sub-component so the top-level file becomes a thin
composition, per the Slint "extract sub-components" pattern used elsewhere in this
settings tree.

- `settings/PlaybackSettings.slint` (~40 lines) — top-level `PlaybackSettings`
  component: the four re-emitted callbacks, imports, and instantiates the three
  group sections below in sequence (with the existing `Divider`/spacer `Rectangle`s
  between them).
- `settings/playback-settings/GroupHeader.slint` (~10 lines) and
  `settings/playback-settings/Divider.slint` (~8 lines) — OR, better, reuse the
  shared `GroupHeader`/`SecondaryButton`-style promoted files from the
  `IntegrationsSettings.slint` split (see that file's plan) if this batch's split
  work happens together — `GroupHeader` and `Divider` here are simple enough to
  share a single common location (e.g. `settings/SettingsShared.slint`) rather than
  being re-duplicated a third time.
- `settings/playback-settings/PlaybackGroup.slint` (~90 lines) — the PLAYBACK section:
  continue-playback, show-context-icon, gapless (disabled-while-streaming-only),
  crossfade slider + label.
- `settings/playback-settings/SessionGroup.slint` (~35 lines) — the SESSION section:
  restore-session, resume-position (disabled unless persist-session).
- `settings/playback-settings/StreamingGroup.slint` (~90 lines) — the STREAMING
  section: stream-uncached, conditional buffer-size slider, streaming-only,
  retry-behavior select.

## Re-export surface
`settings/PlaybackSettings.slint` stays the single import surface — the
`export component PlaybackSettings` signature (4 callbacks) is unchanged; none of the
three group sub-components are exported/imported elsewhere.

## Coupling / watch out
- `GroupHeader`/`Divider` are ALSO duplicated in `IntegrationsSettings.slint` (and
  likely `AppearanceSettings.slint`/`OfflineSettings.slint` per that file's own doc
  comment reference) — this is the clearest cross-file de-duplication opportunity
  in this whole batch: if another agent already promotes these to a shared file
  while splitting `IntegrationsSettings.slint`, this file's split should import that
  same shared file instead of re-creating its own copy. Flag this to whoever performs
  the actual code split.
- The crossfade slider and buffer-size slider both use the same fixed-width
  `HorizontalLayout { width: 200px; ... QbzSlider ... Text }` pattern — could be
  factored into a shared `LabeledSlider` component, but that's a nice-to-have, not
  required for the 130-line split; note it as a follow-up, not part of this plan.
- `gapless`'s `enabled: !SettingsState.streaming-only` and the crossfade row's
  `enabled: !SettingsState.streaming-only` both read the SAME `SettingsState` flag —
  when split into `PlaybackGroup.slint`, both conditions must stay reading
  `SettingsState.streaming-only` directly (no local proxy prop) since Slint's binding
  system handles the cross-reactivity automatically; don't try to "optimize" by
  passing it as a single `in property`.
- `resume-position`'s `enabled: SettingsState.persist-session` in `SessionGroup.slint`
  is a similar direct-global-read pattern — keep it reading `SettingsState` directly.

## Verify after split
- Slint compile check on `PlaybackSettings.slint` and its importer (SettingsView).
- Visual smoke-test: every toggle/slider/select still round-trips through
  `settings-bool`/`settings-select`/`settings-slider`; gapless and crossfade
  correctly grey out when Streaming-only is on; buffer-size slider only shows when
  stream-uncached is on; retry-behavior select still lists options and fires
  `settings-select("retry-behavior", i)`.
