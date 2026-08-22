# `crates/qbz-ui/ui/shell/LinkResolverModal.slint` (290 lines)

"Open Qobuz Link" modal (Ctrl+L): platform-icon + URL input + Go button, error line,
playlist-detected banner handing off to the Playlist Importer.

## Proposed split

- `LinkResolverModal.slint` (~110 lines) — stays the public surface: `export component
  LinkResolverModal`, backdrop, `FocusScope` (Escape-to-close + deferred focus timer),
  card shell, header (title + close X), composes the sub-blocks below.
- `shell/LinkResolverInputRow.slint` (~110 lines) — the platform icon + URL `TextInput`
  box + Go button row (lines ~110-235). Takes no extra props — binds directly to
  `LinkResolverState`/`LinkResolverActions`/`UiFocusState` globals (already globals). Note:
  the `url-input` `TextInput` needs to stay reachable for the parent `FocusScope`'s
  `forward-focus: url-input;` — if this row becomes a separate component, `forward-focus`
  must target the row's exposed input or the row needs a `forward-focus`-compatible export
  (Slint requires the focusable element be visible to the FocusScope in the same
  component tree, so verify this compiles before finalizing the split; may need to keep
  the input inline in the parent instead).
- `shell/LinkResolverPlaylistBanner.slint` (~40 lines) — the playlist-detected banner
  (lines ~246-285).

## Coupling to flag

- **Focus-forwarding risk**: `keys := FocusScope { forward-focus: url-input; ... }` is a
  same-component-tree reference — extracting `url-input` into a child component may break
  `forward-focus` (Slint's forward-focus generally needs a direct-or-exported focus
  target). Verify this compiles; if it doesn't, keep the input row inline and only extract
  the platform-icon-picking logic or the banner.
- The platform icon's long ternary chain (qobuz/spotify/apple/tidal/deezer/fallback) is
  pure display logic tied to `LinkResolverState.platform` — safe to move wherever the
  input row ends up.

## Verify after split

- Slint compile check (pay special attention to `forward-focus` compiling correctly).
- Manual test: Ctrl+L opens with focus in the URL field, Escape closes, paste each
  platform's URL type and confirm the icon updates, Go button state (disabled while
  empty/resolving), error line display, playlist-detected banner + "Open Playlist
  Importer" handoff.
