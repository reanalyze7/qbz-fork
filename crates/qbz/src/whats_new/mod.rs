//! What's New modal controller + release-notes markdown renderer.
//!
//! On open, fetches the GitHub release whose tag matches the running version
//! (`https://api.github.com/repos/vicrodh/qbz/releases/tags/v{version}`) on a
//! worker thread, parses its markdown `body` into a flat block model, and hops
//! back to the Slint event loop to fill `WhatsNewState`.
//!
//! The markdown renderer is a 1:1 port of the Tauri `renderMarkdownWithToc`
//! (`src/lib/utils/markdown.ts`): it supports the same small subset — `#`/`##`
//! headings AND indent-0 `- ` bullets both become level-0 SECTIONS (the TOC),
//! `###` becomes a sub-section, indented bullets nest by `floor(spaces/2)`, and
//! everything else is a paragraph. Inline `**bold**` / `` `code` `` markers are
//! STRIPPED (Slint has no inline rich-text spans in a single Text — accepted
//! deviation; the plain text is preserved).

mod controller;
mod fetch;
mod markdown;

pub use controller::install;

const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/vicrodh/qbz/releases";
