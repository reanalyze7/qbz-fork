//! Block builders shared by the markdown render loop.

use crate::{WhatsNewBlock, WhatsNewTocEntry};

use super::inline::strip_inline;
use super::slug::slugify;
use super::{KIND_LINK, KIND_SECTION};

/// Push a section heading block; level-0 sections also become TOC entries.
pub(super) fn push_heading(
    label: &str,
    level: i32,
    blocks: &mut Vec<WhatsNewBlock>,
    toc: &mut Vec<WhatsNewTocEntry>,
) {
    let clean = label.trim();
    if clean.is_empty() {
        return;
    }
    let id = slugify(clean);
    if level == 0 {
        toc.push(WhatsNewTocEntry {
            id: id.clone().into(),
            label: clean.into(),
        });
    }
    blocks.push(WhatsNewBlock {
        kind: KIND_SECTION,
        level,
        text: strip_inline(clean).into(),
        id: id.into(),
        url: "".into(),
    });
}

/// A clickable whole-line link block.
pub(super) fn link_block(label: &str, url: &str) -> WhatsNewBlock {
    WhatsNewBlock {
        kind: KIND_LINK,
        level: 0,
        text: strip_inline(label).into(),
        id: "".into(),
        url: url.into(),
    }
}
