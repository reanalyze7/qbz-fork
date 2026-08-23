// crates/qbzd/src/tui/screens/wizard/state_query.rs — pure query helpers on
// WizardState (App-facing interface + Review-step selection logic).

use crate::tui::strings as s;

use super::state::WizardState;
use super::step::WStep;

impl WizardState {
    // -------------------------- App-facing interface --------------------------
    //
    // The wizard never edits a persistent store — it is never "dirty" (03: the
    // dirty/save model does not apply; the App treats Wizard as clean, and Esc
    // mid-flow confirms abandon instead).

    /// A field editor (Check override select, or the manual node input) owns the
    /// keyboard — the shell must not steal number keys / focus moves.
    pub fn is_editing(&self) -> bool {
        self.check_editor.is_some() || self.manual.is_some()
    }

    /// The breadcrumb's level-2 node is always the current STEP (`Wizard › …`),
    /// so the operator can see where they are in the flow.
    pub fn editing_label(&self) -> Option<&'static str> {
        Some(self.step.title())
    }

    /// Whether the current step consumes ←/→ itself (step back/next) — true
    /// for every step except Welcome, where nothing in the content claims ←
    /// (the CTA only listens for Enter/Space): there, ← should behave like
    /// every other section and drop focus back to the sidebar instead of
    /// being silently swallowed by a no-op `retreat()`.
    pub fn claims_horizontal(&self) -> bool {
        self.step != WStep::Welcome
    }

    /// Step-specific help bar.
    pub fn help_text(&self) -> &'static str {
        if self.check_editor.is_some() {
            return s::HELP_SELECT;
        }
        if self.manual.is_some() {
            return s::HELP_INPUT;
        }
        match self.step {
            WStep::Welcome => s::WIZ_HELP_WELCOME,
            WStep::Check => s::WIZ_HELP_CHECK,
            WStep::SelectDacs => s::WIZ_HELP_SELECT,
            WStep::Review => s::WIZ_HELP_REVIEW,
            WStep::Test => s::WIZ_HELP_TEST,
            WStep::Done => s::WIZ_HELP_DONE,
        }
    }

    /// The (node_name, display_name) pairs to generate configs for: every checked
    /// candidate, or the manual node when nothing enumerated (1:1 with the Slint
    /// `checked_dacs`).
    pub(super) fn checked_dacs(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = self
            .candidates
            .iter()
            .filter(|c| c.checked)
            .map(|c| (c.id.clone(), c.description.clone()))
            .collect();
        if out.is_empty() {
            if let Some(m) = &self.manual_node {
                out.push((m.clone(), m.clone()));
            }
        }
        out
    }

    /// Whether the Select-DACs step can advance (≥1 checked or a valid manual).
    pub(super) fn has_selection(&self) -> bool {
        self.candidates.iter().any(|c| c.checked) || self.manual_node.is_some()
    }
}
