//! Markdown → flat block model + TOC. Port of the Tauri
//! `renderMarkdownWithToc` (`src/lib/utils/markdown.ts`).

mod builders;
mod inline;
mod slug;

use crate::{WhatsNewBlock, WhatsNewTocEntry};

use builders::{link_block, push_heading};
use inline::{parse_standalone_link, strip_inline};
use slug::count_leading_spaces;

/// Block kinds shared with `WhatsNewBlock.kind` in the Slint model.
const KIND_SECTION: i32 = 0;
const KIND_BULLET: i32 = 1;
const KIND_PARAGRAPH: i32 = 2;
/// A whole-line markdown link `[text](url)` — rendered as a clickable link.
const KIND_LINK: i32 = 3;

/// Render the release-notes markdown into a flat block model + a TOC of the
/// level-0 section headings. 1:1 with `renderMarkdownWithToc`.
pub fn render_markdown(markdown: &str) -> (Vec<WhatsNewBlock>, Vec<WhatsNewTocEntry>) {
    let mut blocks: Vec<WhatsNewBlock> = Vec::new();
    let mut toc: Vec<WhatsNewTocEntry> = Vec::new();

    if markdown.trim().is_empty() {
        return (blocks, toc);
    }

    for line in markdown.split('\n') {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Headings (#, ##, ###).
        if let Some(rest) = trimmed.strip_prefix("# ") {
            push_heading(rest, 0, &mut blocks, &mut toc);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            push_heading(rest, 0, &mut blocks, &mut toc);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            push_heading(rest, 1, &mut blocks, &mut toc);
            continue;
        }

        // List items with indentation-based nesting.
        let is_list = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        if is_list {
            let indent = count_leading_spaces(line);
            let level = (indent / 2) as i32;
            let content = trimmed[2..].trim();

            if level == 0 {
                // Top-level bullets become section headings (no bullet glyph).
                push_heading(content, 0, &mut blocks, &mut toc);
                continue;
            }

            // A bullet that is nothing but a link renders as a clickable link.
            if let Some((label, url)) = parse_standalone_link(content) {
                blocks.push(link_block(label, url));
                continue;
            }

            blocks.push(WhatsNewBlock {
                kind: KIND_BULLET,
                level,
                text: strip_inline(content).into(),
                id: "".into(),
                url: "".into(),
            });
            continue;
        }

        // A paragraph that is nothing but a link renders as a clickable link.
        if let Some((label, url)) = parse_standalone_link(trimmed) {
            blocks.push(link_block(label, url));
            continue;
        }

        // Paragraph.
        blocks.push(WhatsNewBlock {
            kind: KIND_PARAGRAPH,
            level: 0,
            text: strip_inline(trimmed).into(),
            id: "".into(),
            url: "".into(),
        });
    }

    (blocks, toc)
}
