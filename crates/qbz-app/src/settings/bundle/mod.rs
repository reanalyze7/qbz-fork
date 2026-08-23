// crates/qbz-app/src/settings/bundle/mod.rs — the settings portability engine
// (04-settings-portability.md). ONE module, shared by `qbzd` (P0 CLI) and the
// desktop Settings modal (P0/OD7, plan T18): `export(source, opts) -> Bundle`,
// `plan(bundle, target, opts, live) -> ImportPlan`, `apply(plan, target, uid)
// -> ImportReport`.
//
// The load-bearing invariant (04 §1): CLASSIFICATION LIVES IN THE IMPORTER,
// NEVER IN THE BUNDLE. A bundle is data with zero authority over how it is
// applied — the §3 table (the domain loops under `classify/`) is versioned
// with THIS code and applied to whatever is present in the file, including
// fields a well-behaved exporter never writes (a hand-added `volume` is
// skipped no matter how it got there — §1 corollary).
//
// TDD: the inline `#[cfg(test)]` module IS the normative test suite — one test
// per §3/§5 rule. The engine takes a `LiveSystem` by injection so those tests
// never need real audio hardware.

mod apply;
mod apply_writes;
mod apply_writes_audio;
mod classify;
mod error;
mod export;
mod export_misc;
mod plan;
mod plan_types;
mod readers;
mod serde_impl;
#[cfg(test)]
mod tests;
mod token;
mod types;

pub use apply::apply;
pub use error::BundleError;
pub use export::export;
pub use plan::{plan, replan_with_device};
pub use plan_types::{DeviceChoice, DevicePick, ImportPlan, ImportReport, PlanLine};
pub use token::{default_filename, write_bundle_file};
pub use types::{
    Bundle, BundleSource, ExportOptions, ExportSource, ImportOptions, LiveSystem, ProfilePaths,
    SCHEMA_VERSION,
};

#[cfg(test)]
use readers::write_last_user_id;
