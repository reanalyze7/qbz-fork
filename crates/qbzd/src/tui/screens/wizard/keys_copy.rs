// crates/qbzd/src/tui/screens/wizard/keys_copy.rs — the Review step's
// copy/save actions (`c`/`C`/`w`).

use std::time::Instant;

use crate::tui::clipboard;
use crate::tui::strings as s;
use crate::tui::wizard_core;

use super::state::WizardState;

impl WizardState {
    pub(super) fn copy_focused_block(&mut self) {
        let env = self.clip_env;
        if let Some(block) = self.configs.get_mut(self.review_focus) {
            let text = block.data.full_block();
            let report = clipboard::copy(&text, &block.data.short(), &env);
            block.flash = Some((report.tier, Instant::now()));
            self.status_flash = Some((report.detail, Instant::now()));
        }
    }

    /// `C` — copy every block at once. Always ALSO lands a durable file
    /// artifact at a fixed path, independent of which clipboard tier won:
    /// OSC 52 is one-way/unverifiable, so the batch copy must never leave the
    /// operator with nothing to fall back to if the paste silently failed.
    pub(super) fn copy_all_blocks(&mut self) {
        if self.configs.is_empty() {
            return;
        }
        // Prepend the backup command — "copy all" gives the operator a back-up +
        // every DAC's config in one paste.
        let mut parts = vec![format!("# ── back up first ──\n{}", wizard_core::BACKUP_CMD)];
        parts.extend(
            self.configs
                .iter()
                .map(|c| format!("# ── {} ──\n{}", c.data.name, c.data.full_block())),
        );
        let all = parts.join("\n\n");

        let report = clipboard::copy(&all, "all-blocks", &self.clip_env);
        let save = clipboard::write_wizard_file("all-blocks", &all);
        let outcome = match (report.tier, save) {
            // The clipboard chain already fell back to this exact file —
            // don't say "saved" twice.
            (clipboard::Tier::File, Ok(_)) => report.detail,
            (_, Ok(path)) => format!("{} + saved {}", report.detail, path.display()),
            (_, Err(e)) => format!("{} (file save also failed: {e})", report.detail),
        };
        self.status_flash = Some((
            format!("{} ({outcome})", s::wiz_copied_all(self.configs.len())),
            Instant::now(),
        ));
    }

    pub(super) fn write_focused_block(&mut self) {
        if let Some(block) = self.configs.get_mut(self.review_focus) {
            let text = block.data.full_block();
            match clipboard::write_wizard_file(&block.data.short(), &text) {
                Ok(path) => {
                    block.flash = Some((clipboard::Tier::File, Instant::now()));
                    self.status_flash =
                        Some((format!("{} {}", s::WIZ_SAVED_TO, path.display()), Instant::now()));
                }
                Err(e) => {
                    self.status_flash = Some((format!("{}: {e}", s::WIZ_SAVE_FAILED), Instant::now()));
                }
            }
        }
    }
}
