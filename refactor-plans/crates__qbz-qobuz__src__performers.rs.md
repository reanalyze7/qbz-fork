# crates/qbz-qobuz/src/performers.rs (404 lines)

Performer-string parsing (Qobuz `"Name, Role1, Role2 - Name, ..."` format),
role grouping/i18n label lookup, and a large static role-label table.

## Proposed split

- `performers/mod.rs` (~90 lines) — re-export surface, `Performer` struct,
  `parse_performers`, `group_by_role`.
- `performers/roles.rs` (~150 lines) — `role_key`, `humanize_role`,
  `format_role_label`, `group_credits_ordered` (the role-ordering/labeling
  logic).
- `performers/role_labels.rs` (~150 lines) — the static
  `PERFORMER_ROLE_LABELS` table (already isolated at the bottom of the
  current file — pure data, easy lift).
- `performers/tests.rs` (~55 lines) — existing test module.

## Tricky coupling

- `format_role_label` (roles.rs) reads `PERFORMER_ROLE_LABELS`
  (role_labels.rs) — needs `use super::role_labels::PERFORMER_ROLE_LABELS;`.
- `PERFORMER_ROLE_LABELS` is currently `pub(crate)` — keep that visibility.
- The crate doc-example in `parse_performers`'s doctest
  (`use qbz_qobuz::performers::parse_performers;`) must keep resolving —
  re-export `parse_performers` from `mod.rs`.

## Verify after split

`cargo build -p qbz-qobuz`, `cargo test -p qbz-qobuz performers::`
(including the doctest on `parse_performers`).
