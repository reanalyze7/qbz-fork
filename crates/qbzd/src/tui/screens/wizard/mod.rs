// crates/qbzd/src/tui/screens/wizard/mod.rs — the HiFi/DAC Wizard (FB4), the
// setup TUI's SEVENTH section (owner-sanctioned cap break). A six-step
// content-frame flow: Welcome → Check → Select DACs → Review → Test → Done.
//
// The heavy, frontend-agnostic logic is COPIED into `tui/wizard_core.rs` (from
// the Slint `qbz-dac-wizard` crate, which the slint-free daemon must not link).
// This screen owns the transient step state + rendering and asks the App to run
// the blocking probes on a worker (NEVER on the render thread, §5.5). The owner
// emphasis — copyable generated config blocks — is the Review step: one bordered
// block per DAC, `c`/`C` copy, `w` saves under ~/qbzd-wizard/ (never a system
// path). Clipboard tiers live in `tui/clipboard.rs` (SSH-first OSC 52).
//
// Split across sibling files: `step.rs` (the pure step-transition table),
// `state*.rs` (the `WizardState` struct + its non-key/non-draw methods),
// `keys*.rs` (per-step key handling as further `impl WizardState` blocks),
// `draw*.rs` (per-step rendering, ditto). All of `WizardState`'s fields are
// `pub(super)` so every sibling file here can reach them.

mod draw;
mod draw_check;
mod draw_done;
mod draw_review;
mod draw_select;
mod draw_test;
mod keys;
mod keys_check;
mod keys_copy;
mod keys_misc;
mod keys_review;
mod keys_select;
mod state;
mod state_query;
mod state_types;
mod state_worker;
mod step;

pub use state::WizardState;
pub use step::{next_step, prev_step, WStep, STEP_ORDER};

#[cfg(test)]
mod tests_render;
#[cfg(test)]
mod tests_select;
#[cfg(test)]
mod tests_step;
