# crates/qbz-app/src/shell.rs (417 lines)

## Summary
`AppRuntime<A>` — the framework-agnostic composition root for non-Tauri UI
shells (Slint/TUI/headless): owns the `QbzCore`/`RuntimeManager`, builds
with optional visualizer support, and implements minimal per-user session
activation/deactivation (including a guest-profile-adoption ritual on first
real login).

## Proposed split
By responsibility: construction vs. session activation vs. guest-profile
adoption vs. tests. This is a generic `impl<A: FrontendAdapter ...>` block,
so submodules must each carry their own
`impl<A: FrontendAdapter + Send + Sync + 'static> AppRuntime<A> { ... }`
block (Rust allows multiple `impl` blocks for the same type across files as
long as they're in the same crate).

- `shell/mod.rs` (~65 lines) — module doc (lines 1-21), `ActiveSession`
  struct, `AppRuntime` struct definition, and `pub use` re-export of nothing
  extra needed (the type itself lives here since submodules just add impl
  blocks to it via `include!`-free multi-file impls... actually simplest:
  keep `AppRuntime` struct + constructors in `mod.rs` itself since Rust
  requires the struct and at least one impl in the same crate — no
  re-export gymnastics needed, submodules just `impl AppRuntime<A> {}` again).
- `shell/construct.rs` (~90 lines) — `impl AppRuntime`: `with_audio_settings`,
  `new`, `with_visualizer`, `visualizer_tap`, `init`, `core`, `runtime`.
- `shell/session.rs` (~130 lines) — `impl AppRuntime`: `activate_at`,
  `activate`, `activate_offline`, `deactivate`, `is_session_active`,
  `active_user_id`, `with_session_store`.
- `shell/guest_profile.rs` (~35 lines) — `impl AppRuntime`:
  `adopt_guest_profile` (the standalone #553 rename ritual — small and
  self-contained, a good candidate for its own file since it's a distinct
  concern from ordinary activation).
- `shell/tests.rs` (~115 lines) — the entire `#[cfg(test)] mod tests` block,
  declared via `#[cfg(test)] mod tests;` in `mod.rs`.

## Re-export surface
`shell/mod.rs` keeps the `pub struct AppRuntime<A: ...>` definition itself
(not just a re-export) since Rust impl blocks in other files need the type
in scope via `use super::AppRuntime;` or `use crate::shell::AppRuntime;`.
Existing importers (`crate::shell::AppRuntime`, e.g. the Slint shell's
`main.rs`) are unaffected since the path `crate::shell::AppRuntime` is
unchanged — only the struct's impl code moves across files within the
`shell/` directory, which is invisible to external callers.

## Coupling / watch out
- `AppRuntime` is generic over `A: FrontendAdapter + Send + Sync + 'static`
  — every new file's `impl` block must repeat this exact bound; a
  mismatched bound (e.g. missing `Send + Sync`) will fail to compile with a
  confusing "no method found" error rather than a clear bound mismatch, so
  copy the bound verbatim from the original.
- `session: Mutex<Option<ActiveSession>>` (private field) is used by both
  `session.rs` (activate/deactivate/is_session_active/active_user_id/
  with_session_store) — all in one file, no cross-file field access risk.
  `ActiveSession` itself is a private struct — keep it in `mod.rs` next to
  `AppRuntime` since both are defined together and `session.rs` needs to
  construct it.
- `visualizer_tap: Option<VisualizerTap>` field is set in both
  `with_audio_settings` (always `None`) and `with_visualizer` (`Some`) —
  both go in `construct.rs`, no cross-file coupling.
- `adopt_guest_profile` is a private associated fn (not `&self`) called
  only from `activate()` in `session.rs` — needs `use super::AppRuntime;`
  plus the impl-block-splitting caveat above; verify it resolves as
  `Self::adopt_guest_profile(user_id)` still works when the two methods are
  in different files (it will, since both are inherent impls of the same
  type).
- Tests use `qbz_core::NoOpAdapter` and touch `activate_at`, `deactivate`,
  `with_session_store` — after the split, `tests.rs` needs `use super::*;`
  to reach `AppRuntime`, `ActiveSession` is NOT used directly by tests
  (verify) so no extra import needed there.

## Verify after split
- `cargo test -p qbz-app shell::` — all 5 tests
  (`builds_with_explicit_audio_settings`,
  `runtime_state_machine_starts_uninitialized`,
  `core_reports_no_session_before_login`,
  `activate_at_opens_session_and_marks_runtime`,
  `deactivate_clears_session_and_runtime`,
  `with_session_store_round_trips_through_active_session`) green.
- `cargo check -p qbz-app` and grep for `crate::shell::AppRuntime` /
  `qbz_app::shell::AppRuntime` importers (the Slint shell's composition
  root) to confirm the public path is unchanged.
