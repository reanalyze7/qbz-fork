# crates/qbz-app/src/runtime.rs (487 lines)

## Summary
The UI-agnostic runtime session contract (`ADR_RUNTIME_SESSION_CONTRACT.md`):
canonical `RuntimeState`/`RuntimeStatus` state machine, typed `RuntimeError`,
`CommandRequirement` gating enum, the `RuntimeManager` (thread-safe state
holder + transition methods + requirement checks), and `RuntimeEvent`
lifecycle events.

## Proposed split
357 lines over budget. Clean by-domain split: types vs. the manager's
behavior vs. tests.

- `runtime/mod.rs` (~115 lines) — `RuntimeState`, `DegradedReason`,
  `RuntimeStatus` (+ its `Default`), and `RuntimeEvent` — the pure data/enum
  shapes (serde-tagged, no logic beyond `Default` impls).
- `runtime/error.rs` (~55 lines) — `RuntimeError` enum + its `Display` impl
  + `impl std::error::Error for RuntimeError {}`.
- `runtime/requirement.rs` (~20 lines) — `CommandRequirement` enum only
  (small, but conceptually separate from the state machine itself).
- `runtime/manager.rs` (~180 lines) — `RuntimeManager` struct + its full
  `impl` (`new`, `set_queue_source_collection`/`get_queue_source_collection`,
  `get_status`, `set_state`, `set_client_initialized`, `set_legacy_auth`,
  `set_corebridge_auth`, `set_session_activated`,
  `is_bootstrap_in_progress`/`set_bootstrap_in_progress`,
  `check_requirements`, `is_degraded`, `set_degraded`) + its `Default` impl.
  This is the single largest chunk (~185 lines in the source) — if it's
  still over budget after extraction, split `check_requirements` (the
  longest single method, ~50 lines) into its own `runtime/requirement.rs`
  function `check(&RuntimeStatus, CommandRequirement) -> Result<(),
  RuntimeError>` called from `manager.rs`.
- `runtime/tests.rs` (~100 lines, `#[cfg(test)] mod tests`) — unchanged,
  included via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`runtime/mod.rs` re-exports everything currently public: `RuntimeState`,
`DegradedReason`, `RuntimeStatus`, `RuntimeError`, `CommandRequirement`,
`RuntimeManager`, `RuntimeEvent`. Callers use `qbz_app::runtime::{...}` or
`crate::runtime::{...}` unchanged — add `pub use manager::RuntimeManager;
pub use error::RuntimeError; pub use requirement::CommandRequirement;` in
`mod.rs`.

## Coupling / watch out
- `RuntimeManager::set_state` has a big match with per-variant derived-field
  resets — keep the WHOLE match together in `manager.rs`; splitting it
  per-variant would scatter genuinely coupled logic (each arm depends on
  every other for the "what does Ready actually mean" invariant).
- `check_requirements` reads `self.state.read().await` once and matches on
  `req` — if extracted into `requirement.rs`, pass `&RuntimeStatus` in
  (already read) rather than `&RuntimeManager`, to avoid a circular
  `manager.rs` ↔ `requirement.rs` dependency.
- `queue_source_collection_id` is a `RwLock<Option<String>>` field on
  `RuntimeManager` with a long doc comment explaining it tracks
  Mixtape/Collection queue provenance (in-memory only, pending a DB column)
  — keep that doc comment attached to the field in `manager.rs`, it's easy
  to lose context on a plain field move.
- `RuntimeEvent` variants reference `DegradedReason` (defined in `mod.rs`)
  — `runtime/mod.rs` must declare `DegradedReason` before/alongside
  `RuntimeEvent`, or `RuntimeEvent` needs `use super::DegradedReason` if it
  moves to its own file (current plan keeps both in `mod.rs`, avoiding this).

## Verify after split
- `cargo test -p qbz-app runtime` (all 5 existing tests green, including the
  Mixtape-context set/clear test and the four state-machine transition
  tests).
- `cargo check -p qbz-app`
- Grep for `RuntimeManager`/`RuntimeState`/`RuntimeError`/`CommandRequirement`
  importers across the Slint (`qbz`) and any CLI/daemon crates that gate
  commands on runtime state, confirm nothing broke.
