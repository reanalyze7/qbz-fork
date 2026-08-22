# crates/qbz/src/suggestions.rs (426 lines)

## 1. Summary
Immersive "Suggestions" panel controller: assembles recommended tracks
(artist tracks-appears-on + sparse fallback, deterministic shuffle) and
curated playlist/radio cards (book/diamond cover collages) via live Qobuz
artist queries, then applies the result to `SuggestionsState` and builds the
artwork-job list for the panel.

## 2. Proposed module split
By responsibility (constants/types, the deterministic shuffle, the async data
assembly, and the Slint apply/artwork glue):

| New file | Owns | ~lines |
|---|---|---|
| `suggestions/mod.rs` | Module decls + re-exports; module doc comment; the tunable constants (`REC_LIMIT`, `SPARSE_THRESHOLD`, `FALLBACK_LIMIT`, `MAX_PLAYLIST_CARDS`, `BOOK_COVERS`, `RADIO_COVERS`) | ~45 |
| `suggestions/types.rs` | `PlaylistCard`, `SuggestionsPayload` structs, `empty_payload()` | ~40 |
| `suggestions/shuffle.rs` | `splitmix64`, `shuffle_tracks` (deterministic RNG, pure) | ~25 |
| `suggestions/covers.rs` | `track_album_cover`, `track_album_id` (pure helpers) | ~20 |
| `suggestions/load.rs` | `load_suggestions` (the async assembly: artist-detail fetch, rec-track dedupe+fallback+shuffle, curated playlist-card cover harvesting) | ~150 |
| `suggestions/apply.rs` | `playlist_to_card`, `radio_card`, `apply_suggestions`, `set_radio_loading`, `reset_suggestions` (Slint-state glue) | ~110 |
| `suggestions/artwork.rs` | `suggestions_artwork_jobs` | ~45 |

## 3. Re-export / public API surface
`suggestions/mod.rs` re-exports the current public surface used by whichever
view controller drives the immersive panel (likely `main.rs` or a
now-playing/immersive controller):

```rust
mod apply;
mod artwork;
mod covers;
mod load;
mod shuffle;
mod types;

pub use apply::{apply_suggestions, reset_suggestions, set_radio_loading};
pub use artwork::suggestions_artwork_jobs;
pub use load::load_suggestions;
pub use types::{empty_payload, SuggestionsPayload};
```

(`PlaylistCard` is `pub(crate)` today — keep it `pub(crate)` re-exported from
`types.rs`, not part of the external `pub` surface.)

## 4. Tricky coupling / shared-state to watch out for
- `load_suggestions` (async, in `load.rs`) is generic over `AppRuntime<A>` /
  `FrontendAdapter` — this is the one function with the heaviest type bounds
  and the most imports (`qbz_app::shell::AppRuntime`, `qbz_core::FrontendAdapter`);
  keep those imports scoped to `load.rs` only.
- `shuffle_tracks`'s seed is derived in `load_suggestions` as
  `(artist_id << 1) ^ current_track_id.wrapping_add(1)` — this exact formula
  must travel with the call site in `load.rs`, not get "cleaned up" into
  `shuffle.rs` (the seed derivation is a suggestions-specific concern, not
  part of the generic shuffle utility).
- `apply_suggestions` assumes the radio card is always the LAST card
  (playlist cards first, then radio) — `set_radio_loading` (also in
  `apply.rs`) independently re-derives this by searching for `kind ==
  "radio"` with a fallback to "last card". Keep both functions in the same
  file since they encode the same ordering assumption from two angles.
- `suggestions_artwork_jobs` (in `artwork.rs`) recomputes the radio card's
  index as `payload.playlist_cards.len()` — this must stay in sync with
  `apply_suggestions`'s card-ordering; if `apply.rs` and `artwork.rs` are
  split, leave a cross-reference comment in both since neither imports the
  other today.
- `playlist_to_card`/`radio_card` both reference `crate::playlist::to_item`
  indirectly via track mapping in `apply_suggestions` — actually
  `apply_suggestions` itself calls `crate::playlist::to_item` directly for
  `rec_tracks`; make sure that external dependency travels with
  `apply_suggestions` into `apply.rs`.

## 5. What to verify after the real split
- `cargo build -p qbz`.
- Grep for `suggestions::` outside this file to confirm the immersive-panel
  view controller's calls to `load_suggestions`/`apply_suggestions`/
  `reset_suggestions`/`suggestions_artwork_jobs` still resolve.
- Smoke-test in the running app: open the immersive "up next"/suggestions
  panel while a track is playing, verify recommended tracks + playlist cards
  + the radio card render with covers, and that the radio card's loading
  spinner toggles correctly when starting a "Song Radio" session.
