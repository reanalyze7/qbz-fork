# crates/qbz-ui/ui/state/local_library_state.slint — 152 lines

**Verdict: documented exception. Do not split.**

The 22-line overage buys nothing that a split can recover, and every
mechanism that could recover it costs either a hard Slint constraint
violation or a rename across 84 files. The argument follows.

---

## 1. Why is this file long?

### What it actually contains

| Kind | Count | Lines |
|---|---|---|
| `export global LocalLibraryState` | 1 | 8–152 |
| struct definitions | 0 | — |
| companion Actions global | 0 (already split out) | — |
| `import` lines | 4 | 2–5 |
| property declarations | 86 | — |
| comment-only lines | 53 | — |
| blank lines | 7 | — |
| header / braces | 6 | — |

The file is **one `global` block and nothing else**. The four imported
types (`DiscoverSection`, `AlbumCardItem`/`AlphaJump`,
`EphemeralAlbum`/`FolderNode`/`FolderSubcardItem`/`LocalArtistItem`/
`LocalArtistSection`, `TrackItem`) are all already defined elsewhere —
`state/cards_extra.slint`, `state/cards_misc.slint`,
`state/drag_local_structs.slint`, `state/track_item.slint`. The
companion `LocalLibraryActions` global was already moved out to
`state/local_library_actions.slint` (102 lines, under budget).

So the two usual escape hatches for an oversized Slint state file —
"hoist the structs" and "hoist the Actions global" — **have already been
taken**. There is nothing movable left that is not a property of the
global itself.

### The hard constraint

A Slint `global` is a single atomic declaration. Its properties cannot
be spread across files, cannot be inherited, and cannot be composed from
a mixin. `export global X { ... }` either appears once in one file or it
does not exist. This is not a style preference in the Slint compiler; it
is the shape of the AST node. Therefore the only way to reduce this file
is to reduce the number of properties in `LocalLibraryState` — which
means moving them into a *different, differently-named* global, which
means renaming the access path at every call site.

`crates/qbz-ui/ui/state.slint` (the barrel) already records this
conclusion at lines 10–15, alongside the same verdict for
`state/appearance_state.slint` (197 lines). This plan is the detailed
justification the barrel comment points at.

---

## 2. The seams that exist (and why they are not worth cutting)

The file is internally organised by browse tab. The seams are real and
clean; the problem is what crosses them.

| Section | Lines | Props | Candidate global |
|---|---|---|---|
| Tab selector + header stats | 8–17 | 7 | (would have to stay shared) |
| Albums tab | 19–51 | 18 | `LocalLibraryAlbumsState` |
| Tracks tab | 53–73 | 14 | `LocalLibraryTracksState` |
| Folders tab (flat + tree + detail) | 75–115 | 26 | `LocalLibraryFoldersState` |
| Ephemeral folder session | 117–128 | 7 | `LocalLibraryEphemeralState` |
| Artists tab (master/detail) | 130–151 | 14 | `LocalLibraryArtistsState` |

A six-way split would land every file at roughly 25–50 lines. That is
the *only* thing it would achieve.

### What crosses the seams

**(a) Cross-tab Rust helpers.** `crates/qbz/src/local_library/shared.rs`
`reset_browse_models()` clears `albums`, `folders`, `tracks` and
`artists` through one handle:

```rust
let s = window.global::<LocalLibraryState>();
s.set_albums(empty_albums.clone());
s.set_folders(empty_albums);
s.set_tracks(empty_tracks);
s.set_artists(...);
```

After a split this becomes four `window.global::<...>()` lookups in one
function — the same code, longer, with the four-tab invariant no longer
visible in one place.

**(b) Preference persistence.** `crates/qbz/src/locallibrary_prefs.rs`
loads and saves `tracks_group_mode`, `tracks_sort` and `albums_id_mode`
in two adjacent three-line blocks (lines 70–72 and 92–94). These are one
persistence unit — `locallibrary_ui.json` — spanning two proposed
globals.

**(c) Chrome / nav flags.** `crates/qbz/src/nav_flags_chrome/part2.rs`
and `part3.rs` decide the selection-bar and back-button behaviour by
reading `active-tab`, `albums-multi-select`, `tracks-multi-select` and
`tracks-visible` together. `active-tab` belongs to the shared header;
the multi-select flags belong to two different tabs. Every one of these
predicates would need two or three handles.

**(d) The artwork pipeline.** `crates/qbz/src/artwork/target.rs`
documents four `ArtworkTarget` variants that address rows *by global +
model + index*: `LocalLibraryState.folders[i]`,
`.folder-detail-subfolders[i]`, `.artists-selected-albums[i]`,
`.artists[i]`. `crates/qbz/src/artwork/apply/library_local.rs` resolves
all four through a single `window.global::<crate::LocalLibraryState>()`.
Splitting spreads one dispatch function across three globals, and the
doc comments in `target.rs` (the map of what an artwork index means)
stop naming a real type.

**(e) Slint-side compound conditions.** `locallibrary/LibToolbarRow.slint`
gates on `active-tab == "folders" && !ephemeral-active`;
`locallibrary/LibTabContent.slint` fans out on `active-tab` alone. These
survive a split but need one extra `import` line per file, in 32 files.

### The cost, measured

- **253** `LocalLibraryState` references across **32** `.slint` files.
- **145** references across **52** Rust files (38 under
  `crates/qbz/src/local_library/`, plus `artwork/apply/library_local.rs`,
  `artwork/target.rs`, `locallibrary_prefs.rs`, `main.rs`,
  `nav_flags_chrome/part2.rs`, `part3.rs`,
  `navigate_recent_library/part5.rs`, `row_toggles/part3.rs`,
  `tag_editor/refresh.rs`, `wire_local_library_settings/part6.rs`,
  `part9.rs`, `part11.rs`, `part12.rs`, `wire_search/part6.rs`).
- The generated Rust bindings change name for **86** getter/setter pairs.
  Slint generates these from the global name, so `set_albums_sort` on
  `LocalLibraryState` becomes `set_albums_sort` on
  `LocalLibraryAlbumsState` — a type change at every call, not a
  mechanical string substitution.
- `crates/qbz-ui/ui/app.slint:50` re-exports `LocalLibraryState` in a
  single flat list; it would carry six names instead of one.

Roughly 400 edit sites, ~84 files touched, to move 22 lines.

---

## 3. Rejected alternatives

### 3a. Split into per-tab globals (six files)

Rejected on the cost above. It also breaks the property that makes this
file readable: the local library is *one screen with four tabs sharing a
header and a load lifecycle*, and the code that manages it
(`crates/qbz/src/local_library/`) is already correctly decomposed into
`albums/ artists/ ephemeral/ folders_flat/ folders_tree/ tracks/`. The
Rust side has the per-tab modularity. The Slint global is the shared
contract those modules write into; fragmenting the contract to mirror
the modules would not add structure, it would remove the one place where
the whole surface is visible.

### 3b. Trim the comments

53 of 152 lines are comment-only, plus ~20 trailing inline comments.
Deleting 23 comment lines would put the file under 130 with no code
change. Rejected: these comments are the only record of the enum-string
domains (`albums-sort` accepts six exact strings, `tracks-sort` eight,
`albums-id-mode` two), of which properties are the artwork pipeline's
index target versus which are derived render sets (`albums` vs
`albums-visible` vs `albums-grouped` — a distinction you cannot recover
from the type, since all three are `[AlbumCardItem]`), and of decisions
with dates and issue numbers attached (`albums-id-mode` / #411, tree as
the default per the 2026-06-06 owner call). Slint has no type-level
enums here; the comment *is* the type. Trading it for a line count
inverts the point of the 130-line rule, which exists to keep files
comprehensible.

### 3c. Move the prose into `state/README.md`

A middle path: keep one-word markers in the file, move the paragraphs to
a package README. Rejected for the string-domain comments specifically —
a `// off | alpha | artist` next to the property is checked when the
property is edited; the same sentence in a README is not. It is worth
doing for the *narrative* blocks (the paging strategy at lines 19–23,
the network-folder rationale that already lives in `shared.rs`), but
that recovers maybe 8 lines and leaves the file at ~144. It does not
reach 130, so it does not change the verdict; treat it as optional
tidying, not a split.

---

## 4. What the public surface stays

Unchanged. `crates/qbz-ui/ui/state.slint:42` keeps

```
export { LocalLibraryState } from "state/local_library_state.slint";
```

and `crates/qbz-ui/ui/app.slint:50` keeps `LocalLibraryState` in its
re-export list. No importer changes. No Rust `use crate::LocalLibraryState`
changes.

---

## 5. Action items

1. No source file is modified by this plan.
2. Keep the exception note in `crates/qbz-ui/ui/state.slint` lines 10–15;
   add a pointer to this document so the next audit pass finds the
   reasoning instead of re-deriving it.
3. Optional, unrelated to the budget: add
   `crates/qbz-ui/ui/state/README.md` describing what each `state/*.slint`
   file owns (the barrel's inline table is the seed). This is the rule-3
   package-README obligation, which the `state/` directory currently does
   not satisfy for any of its 58 files.
4. If `LocalLibraryState` grows further, the first thing to move is not a
   property but the *next* Actions-shaped thing — new callbacks belong in
   `state/local_library_actions.slint`, and new row/card shapes belong in
   `state/drag_local_structs.slint` or `state/cards_misc.slint`. Keep
   those two escape hatches open.

---

## Risks

- **Risk of the recommended non-action:** the file stays 22 lines over
  budget, so an automated line-count gate (if one is ever added to CI)
  fails on it. Mitigation: the gate needs an allowlist, and this document
  plus `state/appearance_state.slint`'s equivalent are its two entries.
- **Risk of drift:** with no size pressure, the global can accrete
  properties indefinitely. The 86 properties are currently one coherent
  screen; at ~120 they would not be. Mitigation: action item 4 — route
  new declarations to the already-split neighbours, and re-open this plan
  if the property count passes 100.
- **Risk if someone splits it anyway:** the failure mode is silent, not a
  compile error. Slint globals are default-initialised, so a Rust call
  site left pointing at the old global after a partial migration compiles
  only if the old global still exists — and if it does, it reads a
  default value instead of the live one. The Albums/Folders `albums` vs
  `folders` pair (both `[AlbumCardItem]`, both artwork targets) is the
  most likely place for a mis-routed setter to produce a blank grid
  rather than an error. Any future split must be all-at-once, with the
  old global deleted in the same commit so every stale reference fails to
  compile.
- **Risk of the optional README (item 3):** a file-by-file table in a
  README goes stale as `state/` files are split further. Mitigation:
  describe domains, not filenames, and let the barrel's export lines
  remain the authoritative file map.
