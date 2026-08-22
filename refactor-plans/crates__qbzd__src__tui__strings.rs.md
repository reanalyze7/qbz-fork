# crates/qbzd/src/tui/strings.rs (422 lines)

## Summary
Every user-facing string constant/fn for the `qbzd setup` TUI, centralized
per-screen (Account, Audio, Playback, Network, Import/Export, HiFi Wizard,
Scrobbler) so a later gettext/i18n pass is a batch job rather than a
rewrite; already organized with `// ==== Section ====` banner comments
matching the TUI's own screen names.

## Proposed split
Pure data (all `pub const &str` / tiny `pub fn -> String` formatters, zero
logic beyond `format!`) — split one file per screen/section, exactly along
the file's existing banners:

- `strings/mod.rs` (~10 lines) — module doc (adapted from lines 1-5) +
  `pub use` re-export of every submodule's constants/fns.
- `strings/shell.rs` (~100 lines) — lines 7-115: entry/guard strings
  (`NON_TTY_ERROR`, `too_small`), shell/navigation (`APP_TITLE`,
  `HELP_TITLE`, `BREADCRUMB_ROOT`, `SIDEBAR_LABELS*`, `SIDEBAR_SUMMARIES`,
  all `HELP_*` constants, `HELP_OVERLAY`).
- `strings/dirty_footer.rs` (~20 lines) — lines 102-116: dirty-save/quit
  (`DIRTY_TITLE`, `DIRTY_BODY`, `DIRTY_HINT`) + footer (`FOOTER_*`,
  `APPLIES_ON_START`).
- `strings/account.rs` (~35 lines) — lines 117-148: all `ACCOUNT_*`
  constants + `account_logged_in`/`account_logged_in_plan` fns +
  `CONFIRM_YN`.
- `strings/audio.rs` (~50 lines) — lines 150-196: all `AUDIO_*`, `A_*`,
  `R_ALSA_ONLY`/`R_PIPEWIRE_ONLY`/`R_PASSTHROUGH_OFF`, `DSD_*`, `ALSA_*`,
  `AUDIO_SCANNING`, `DEVICE_PICKER_TITLE`, `NO_DEVICES`, `JACK_WARNING`,
  `BP_BADGE`.
- `strings/playback.rs` (~45 lines) — lines 198-237: all `PLAYBACK_*`,
  `P_*`, `R_LIMIT_OFF`/`R_STREAMING_ONLY_ON`/`R_RESTORE_OFF`, `Q_*`,
  `RETRY_*`, `AUTOPLAY_*`, `RATE_NO_LIMIT`.
- `strings/network.rs` (~25 lines) — lines 239-264: all `NETWORK_*`, `N_*`.
- `strings/bundle.rs` (~45 lines) — lines 266-302: all `BUNDLE_*`, `B_*`,
  `b_export_success`, `b_import_done`.
- `strings/save_result.rs` (~10 lines) — lines 304-312: `SAVE_TITLE`,
  `RESULT_HINT`, `SAVED_DISK_ONLY`, `RELOAD_REFUSED`.
- `strings/wizard.rs` (~100 lines) — lines 314-409: all `WIZARD_*`,
  `WIZ_*` constants and fns (`wiz_sandbox_note`, `wiz_copied_all`,
  `wiz_done_summary`), `WIZ_ABANDON_*`. This is the single largest
  section — if it lands over 130 with re-exports/doc comments, split into
  `wizard/steps.rs` (step names + per-step help bars + welcome/check/select
  text) and `wizard/review_test_done.rs` (review/test/done step text +
  abandon modal).
- `strings/scrobbler.rs` (~15 lines) — lines 411-421: `SCROBBLER_TITLE`,
  `HELP_SCROBBLER`, `SCROBBLE_*_HANDOFF`, `SCROBBLE_RETURN_HINT`.

## Re-export surface
`strings/mod.rs` re-exports every submodule's public items
(`pub use shell::*; pub use account::*; pub use audio::*; ...`) so every
existing call site in the `tui/` screens keeps writing
`strings::ACCOUNT_TITLE`, `strings::wiz_sandbox_note(...)`, etc. — i.e. the
public path stays `crate::tui::strings::X` exactly as today, only the
physical file backing each constant changes.

## Coupling / watch out
- This file is pure data with essentially zero coupling to anything besides
  `std::format!` — the ONLY risk in this split is typos in constant names
  or accidentally dropping a doc comment that documents a spec section
  reference (e.g. "03 §3.5", "FB4", "FB5") — these doc comments are
  deliberate traceability back to the design doc and should be preserved
  verbatim when moving each constant.
- A few constants cross-reference sibling sections in their doc comments
  (e.g. `SIDEBAR_SUMMARIES` numbering matches `SIDEBAR_LABELS`/
  `SIDEBAR_LABELS_WIDE` order 1:1, and `R_STREAMING_ONLY_ON` in
  `playback.rs` textually references "Audio > Streaming only" which lives
  in `audio.rs`) — no code coupling, just keep the numbering/order
  consistent across `SIDEBAR_LABELS`, `SIDEBAR_LABELS_WIDE`, and
  `SIDEBAR_SUMMARIES` if they end up in different files (recommend keeping
  all three together, they already are in `strings/shell.rs`).
- No `#[cfg(test)]` module in this file — verification is compile +
  visual/manual TUI check only.

## Verify after split
- `cargo check -p qbzd` (this is a leaf data file consumed only by
  `crate::tui::*` screen-rendering code within the same crate).
- Run `qbzd setup` interactively (or the project's TUI smoke-test skill, if
  one exists) and spot-check a few screens (Account, Audio, Wizard) to
  confirm every string still renders — since there's no automated test
  coverage for these constants, a compile pass alone won't catch a
  copy-paste truncation or an accidentally-dropped multi-line string.
