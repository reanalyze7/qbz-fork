# crates/qbz/src/whats_new.rs (412 lines)

## Summary
What's New modal: fetches the matching GitHub release for the running
version, and a from-scratch markdown → flat-block-model + TOC renderer
(1:1 port of the Tauri `renderMarkdownWithToc`).

## Proposed split
Split cleanly along the file's own `// ==== Markdown -> blocks + TOC
====` banner — fetch/controller vs renderer are fully independent:

- `whats_new/mod.rs` (~15 lines) — `pub use` of `controller` and
  `markdown`.
- `whats_new/controller.rs` (~135 lines) — `GITHUB_RELEASES_URL`, the
  `KIND_*` consts (shared with the renderer, so re-export or duplicate —
  see coupling note), `GithubRelease`, `FetchedRelease`, `install`, `apply`,
  `fetch_release_for_version`, `normalize_version_tag`,
  `format_release_date`. Slightly over — split `fetch_release_for_version`
  + its two helpers (~70 lines) into `whats_new/fetch.rs`, leaving `install`
  + `apply` (~65 lines) in `controller.rs`.
- `whats_new/markdown/mod.rs` (~50 lines) — `strip_inline`,
  `strip_markdown_links`, `parse_link_at`, `parse_standalone_link` (inline
  markup handling), `pub use` of `render`.
- `whats_new/markdown/slug.rs` (~30 lines) — `slugify`,
  `count_leading_spaces`.
- `whats_new/markdown/render.rs` (~90 lines) — `push_heading`,
  `link_block`, `render_markdown` (the main line-by-line state machine).

## Re-export surface
`whats_new/mod.rs` stays the `mod whats_new;` target. `install` (called
once from shell setup) and `render_markdown` (used only internally by
`apply`, but keep `pub` if any test or other caller needs it) re-exported
via `pub use controller::install; pub use markdown::render_markdown;` so
`crate::whats_new::install` is unchanged.

## Coupling / watch out
- The `KIND_SECTION`/`KIND_BULLET`/`KIND_PARAGRAPH`/`KIND_LINK` consts are
  used by BOTH the markdown renderer (`render.rs`) and match the Slint
  `WhatsNewBlock.kind` contract documented in the module doc comment —
  define them once (in `markdown/mod.rs`, since that's their true owner)
  and have `render.rs` `use super::{KIND_SECTION, ...};`; `controller.rs`
  doesn't need them at all once split.
- `apply` (controller.rs) calls `render_markdown` (markdown/render.rs) —
  needs `use super::markdown::render_markdown;` from `mod.rs`'s re-export,
  or a direct `use crate::whats_new::markdown::render_markdown;`.
- The renderer's doc comment stresses it's a "1:1 port" of a specific
  TypeScript function (`renderMarkdownWithToc`) with a documented, narrow
  supported subset (headings, indent-0 bullets as sections, `###` as
  sub-section, inline bold/code STRIPPED not rendered) — preserve this
  exact behavior; do not "improve" the markdown support as a side effect of
  moving files.
- `parse_link_at`'s doc comment notes "ASCII... delimiters... all returned
  slices sit on char boundaries" — a correctness-critical assumption for
  the byte-index arithmetic; keep the whole fn (not split further) and its
  comment together.

## Verify after split
- `cargo build -p qbz` (no `#[cfg(test)]` in this file — flag as a gap;
  the markdown renderer especially is dense enough to deserve unit tests
  ported from the Tauri original in a real split PR).
- Smoke-test: open What's New modal, confirm the release fetch + rendered
  TOC/sections/bullets/links match a known GitHub release body.
