# `crates/qbz-ui/ui/shell/AboutModal.slint` (531 lines)

About QBZ modal: branding header, description, Qobuz legal notice, external-links row,
build-info grid, acknowledgments, author + contributors chips, signature footer.

## Proposed split

- `AboutModal.slint` (~90 lines) — stays the public surface: `export component
  AboutModal`, backdrop, card shell, header (branding + close X), the scrolling
  `Flickable` wrapper composing the body sections below.
- `shell/LinkButton.slint` (~40 lines) — extract the internal `LinkButton` component
  (lines 23-72).
- `shell/HandleChip.slint` (~50 lines) — extract the internal `HandleChip` component
  (lines 77-138), used by both the author row and contributors flow.
- `shell/SectionHeading.slint` (~10 lines) — trivial extract of the `SectionHeading`
  text-style component (lines 141-146).
- `shell/AboutBody.slint` (~260 lines, still needs its own internal split — see below) —
  the entire scrolling body content (description, legal notice, links row, build-info
  grid, acknowledgments, author, contributors, signature). Even after extracting the three
  small components above, this body content is still large; split it further into:
  - `shell/AboutBuildInfo.slint` (~90 lines) — the two-column build-info grid
    (version/license/build + codename/platform).
  - `shell/AboutContributors.slint` (~50 lines) — author chip + `contributor-rows` flow.
  - `shell/AboutSignature.slint` (~50 lines) — the "Made with ~~love~~ hate in [flag]"
    footer paragraph.
  - The remaining description/legal-notice/links-row content (~70 lines) stays directly
    in `AboutBody.slint` (or inline in `AboutModal.slint` if that keeps it under budget).

## Coupling to flag

- All content binds directly to the `AboutState`/`AboutActions` globals — no
  prop-threading required for any of the extracted body sections.
- `AboutState.contributor-rows` is pre-grouped into fixed rows of 4 by the Rust side
  (`crate::about`, per the comment) — a Slint-side flex-wrap workaround; keep the nested
  `for group ... for ack in group.items` loop structure intact wherever it moves.

## Verify after split

- Slint compile check.
- Visual smoke test: modal opens/closes, external links open the right URLs, build-info
  values render correctly, author + contributor avatar chips load and open profiles,
  signature renders (flag image, strikethrough "love").
