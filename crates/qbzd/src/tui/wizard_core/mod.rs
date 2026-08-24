// crates/qbzd/src/tui/wizard_core/ — the frontend-agnostic HiFi-wizard logic.
//
// COPIED from crates/qbz-dac-wizard/src/lib.rs @ 7678bceb (the Slint controller)
// TODO(converge: dac-wizard) — fold into a shared crate (P1). The Slint crate
// depends on `slint` + `qbz-ui` (it drives `DacWizardState` globals), which the
// slint-free `qbzd` column must NOT link. So the window-free PURE pieces are
// copied here and the imperative `DacWizardState` plumbing (open_immediate /
// apply_health / recompute / apply_candidates / apply_configs / apply_poll / …)
// is re-expressed as plain data the TUI screen owns.
//
// Adaptations vs the original (all minimal, none touch the emitted config text):
//   1. `qbz_i18n::t("…")` / `tf(…)` calls dropped — the setup TUI is English-only
//      (03 §1.2), so caption literals are inlined (`.to_string()`); the plural
//      summary line is built by the screen, not here.
//   2. `DacCandidateData` / `DacConfigData` fields made `pub` so the sibling
//      screen module can read them (the original read them within one module).
//   3. `DacConfigData` gains `full_block()` / `target_paths()` / `short()` so the
//      TUI can render/copy/save one bordered box per DAC (the Slint accordion had
//      three sub-blocks + a separate created-paths list).
//   4. `seed_for_rate_depth()` added as the non-test call site for the copied
//      `track_matches_seed()` (the Slint side called it from the search-resolve
//      path, which the daemon test step does not reproduce).
// Everything else — remediation/reference command generation, the three config
// generators, slugify/short_name/rates_list, the test seeds — is verbatim.
mod config_gen;
mod config_templates;
mod detect;
mod remediate;
mod remediate_pkgs;
mod test_seeds;
#[cfg(test)]
mod tests;

pub use config_gen::{gen_configs_blocking, DacConfigData, BACKUP_CMD};
pub use detect::{detect_blocking, detect_dac_type, validate_node_name, DacCandidateData};
pub use remediate::{reference_commands, remediations, restart_cmd};
pub use test_seeds::{khz, negotiated_label, seed_for_rate_depth, TEST_SEEDS};
