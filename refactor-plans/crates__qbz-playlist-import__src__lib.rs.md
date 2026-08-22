# crates/qbz-playlist-import/src/lib.rs (158 lines)

## 1. Summary

The crate root: declares the `errors`/`importer`/`match_qobuz`/`models`/
`providers`/`sink`/`http` sub-modules, re-exports their public types, adds
one crate-level constant (`QBZ_PROXY_BASE`), and defines a small
`ProviderKey` enum + `detect_provider_key` (a looser UI-facing provider
detector mirroring the frontend's `detectProvider`) with its own test
table.

## 2. Proposed module split

The barrel/re-export portion (lines 1-45) is already minimal and correct
as a `lib.rs` — the overage comes entirely from the `ProviderKey` type +
`detect_provider_key` function + its tests, which are a self-contained
mini-feature that doesn't need to live in the crate root at all:

| New file | Owns | ~lines |
|---|---|---|
| `lib.rs` | Crate doc comment, `mod`/`pub use` declarations, `QBZ_PROXY_BASE` constant, `mod provider_key;` + `pub use provider_key::{ProviderKey, detect_provider_key};` | ~50 |
| `provider_key.rs` | `ProviderKey` enum + `impl ProviderKey::as_str`, `detect_provider_key` function | ~65 |
| `provider_key/tests.rs` (or `#[cfg(test)] mod tests` inline at the bottom of `provider_key.rs`, since it's already under 130 with tests included — see §4) | The 3 existing tests | ~65 |

Actually, `provider_key.rs` (enum + function, ~65 lines) plus its tests
(~65 lines) fits comfortably in one ~130-line file — no further split
needed. The two-file table above (`lib.rs` + `provider_key.rs`) is
sufficient to bring `lib.rs` itself under 130 lines; keep
`provider_key.rs`'s tests inline via `#[cfg(test)] mod tests` at its
bottom rather than adding a third file, since that keeps it at ~130 lines
total and matches how the sibling `errors`/`importer`/etc. modules likely
structure their own tests (check their convention before deciding).

## 3. Re-export / public API surface

`lib.rs` remains the single import surface: `qbz_playlist_import::{
PlaylistImportError, import_public_playlist, preview_public_playlist,
ImportPlaylist, ImportProgress, ImportProvider, ImportSummary,
ImportTrack, TrackMatch, detect_music_resource, MusicProvider,
MusicResource, ImportEvent, ImportPhase, ImportProgressSink, QBZ_PROXY_BASE,
ProviderKey, detect_provider_key }` — every one of these paths keeps
resolving unchanged; only `ProviderKey`/`detect_provider_key`'s
*definition* moves to `provider_key.rs`, re-exported via `pub use
provider_key::{ProviderKey, detect_provider_key};` in `lib.rs`.

## 4. Tricky coupling to watch out for

- `detect_provider_key`'s doc comment explicitly says it mirrors the
  Svelte frontend's `detectProvider` and is deliberately **looser** than
  `providers::detect_provider` (the backend's authoritative validator) —
  this relationship/contrast is important context; carry the comment
  into `provider_key.rs` verbatim so it isn't read in isolation without
  the "these two functions can disagree, that's intentional" caveat.
- `QBZ_PROXY_BASE` constant's doc comment notes it was "Hoisted from the
  Tidal provider so the future link-resolver port shares the one
  constant instead of duplicating it" — this stays in `lib.rs` since it's
  crate-wide infrastructure, not part of the `ProviderKey` feature; don't
  accidentally move it into `provider_key.rs` just because it's
  physically adjacent in the current file.
- The crate's top doc comment (the big block about known provider
  limitations — Spotify/Deezer/Apple/Tidal caveats) belongs to the crate
  root (`lib.rs`) as a whole, not to any one sub-module; keep it there.

## 5. What to verify after the real split

- `cargo build -p qbz-playlist-import` and `cargo test -p
  qbz-playlist-import` — all 3 existing tests stay green
  (`detect_provider_key_table`, `detect_provider_key_trims_whitespace`,
  `provider_key_as_str`).
- Grep the workspace for `qbz_playlist_import::ProviderKey` and
  `qbz_playlist_import::detect_provider_key` usages (likely the UI's
  import-playlist modal gating logic) to confirm the re-export keeps
  those call sites compiling.
- Smoke-test the playlist importer UI: pasting a Spotify/Apple/Tidal/
  Deezer playlist URL still enables the Import button correctly (the
  `detect_provider_key` UI gate), and an actual import still runs
  end-to-end via `import_public_playlist`.
