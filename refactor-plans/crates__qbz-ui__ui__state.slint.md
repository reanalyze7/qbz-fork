# crates/qbz-ui/ui/state.slint (5,126 lines)

## 1. Summary

One monolithic Slint file holding ~90 `export global` singletons (State +
Actions callback surfaces) and ~50 `export struct` DTOs that back every
feature area of the app (Home, Discover, Local Library, Sidebar, MyQbz,
Settings/Appearance, Artist/Label, Shell/NowPlaying/Queue, Search,
Favorites, Offline, Dac Wizard, Keybindings, About, What's New, etc.).
~150+ call sites across the UI do `import { X } from "state.slint";` (or a
relative path to it), so the split must preserve `state.slint` as a
100%-compatible import surface.

## 2. Split strategy — one (or a few tightly-coupled) global(s)/struct(s) per file

Slint supports `export { X } from "path";` re-export syntax, so
`state.slint` becomes a **thin barrel**: every symbol it currently defines
is instead defined in a new file under `crates/qbz-ui/ui/state/`, and
`state.slint` re-exports all of them. No import path outside `state.slint`
itself needs to change.

New directory: `crates/qbz-ui/ui/state/`. Proposed files (grouped by the
domain the code already clusters into; approximate line spans computed
from the current file's boundaries — most land at 60-130 lines, a few
exceed 130 because a *single* `global`/`struct` block is itself large and
Slint has no way to split one global across files):

| New file | Globals/structs it owns | ~lines |
|---|---|---|
| `state/cards_misc.slint` | `AlbumCardItem`, `AlphaJump`, `BulkAction`, `ArtistReleaseSection`, `DiscoverSection`, `SlimItem`, `SearchPlaylistItem`, `PinnedItem` | 133 |
| `state/pinned_home.slint` | `PinnedState`, `PinnedActions`, `PlaylistTagItem`, `CortinillaRow`, `CortinillaSection`, `HomeState`, `HomeActions` | 125 |
| `state/discover_home.slint` | `RecentAlbumsState`, `MostPlayedAlbumsState`, `MostPlayedAlbumsActions`, `SectionDescriptor`, `ConfigRow`, `DiscoverState`, `DiscoverActions` | 83 |
| `state/discover_browse.slint` | `DiscoverBrowseState`, `DiscoverBrowseActions`, `PlaylistBrowseState`, `PlaylistBrowseActions`, `ForYouState` | 109 |
| `state/discover_reco_genre.slint` | `ExternalRecoState`, `ExternalRecoActions`, `GenreChip`, `GenreTreeRow`, `GenreFilterState`, `GenreFilterActions`, `TextUtil` | 113 |
| `state/track_item.slint` | `TrackItem`, `ArtistCredit` | 81 |
| `state/album_state.slint` | `AlbumState` (single global, already 106 lines) | 106 |
| `state/track_info.slint` | `InfoCreditRow`, `InfoCreditPair`, `AlbumCreditPerformer`, `AlbumCreditTrack`, `TrackInfoState`, `TrackInfoActions` | 87 |
| `state/suggestions.slint` | `SuggestionCard`, `SuggestionsState`, `SuggestionsActions`, `PlaylistSuggestionRow`, `PlaylistSuggestionsState`, `PlaylistSuggestionsActions` | 122 |
| `state/album_info_booklet.slint` | `AlbumInfoState`, `AlbumInfoActions`, `AlbumActions`, `BookletState`, `BookletActions` | 95 |
| `state/drag_local_structs.slint` | `DragState`, `DragActions`, `LocalArtistItem`, `LocalArtistSection`, `FolderNode`, `FolderSubcardItem`, `EphemeralAlbum` | 95 |
| `state/local_library_state.slint` | `LocalLibraryState` (single global — **146 lines, exceeds 130**) | 146 |
| `state/local_library_actions.slint` | `LocalLibraryActions` (single global, 103) | 103 |
| `state/local_album.slint` | `LibAlbumFilterState`, `LocalAlbumVersion`, `LocalAlbumState`, `LocalAlbumActions`, `TrackMenuState` | 111 |
| `state/library_folders_scrobble.slint` | `LibraryFolderItem`, `LibraryFoldersState`, `LibFolderEditState`, `LibraryScanState`, `LibraryManageActions`, `ScrobbleState` | 103 |
| `state/scrobble_tag_editor.slint` | `ScrobbleActions`, `TagTrackEdit`, `RemoteResultItem`, `TagEditorState`, `TagEditorActions`, `ToastState` | 105 |
| `state/sidebar_structs_popups.slint` | `SidebarPlaylistItem`, `SidebarEntry`, `SidebarFolderPopupState`, `SidebarTooltipState`, `SidebarPlaylistsPopupState` | 75 |
| `state/sidebar_state_actions.slint` | `SidebarState`, `SidebarActions` | 67 |
| `state/create_playlist_folder_myqbz.slint` | `CreatePlaylistState`, `CreatePlaylistActions`, `CreateFolderState`, `CreateFolderActions`, `SettingsExportState`, `SettingsExportActions`, `MyQbzCreateState`, `MyQbzCreateActions` | 82 |
| `state/myqbz_add_ephemeral.slint` | `MyQbzAddRow`, `MyQbzAddState`, `MyQbzAddActions`, `EphemeralPlayChoiceState`, `EphemeralPlayChoiceActions` | 81 |
| `state/playlist_state.slint` | `PlaylistState`, `PlaylistActions`, `EditPlaylistState`, `EditPlaylistActions` | 90 |
| `state/playlist_manager_structs.slint` | `PmFolderItem`, `PmPlaylistItem`, `PmTreeRow` | 67 |
| `state/playlist_manager_state.slint` | `PlaylistManagerState`, `PlaylistManagerActions` | 66 |
| `state/offline_manager.slint` | `OfflineRow`, `OfflineArtist`, `OfflineManagerState`, `OfflineManagerActions` | 73 |
| `state/blacklist.slint` | `BlacklistedArtistItem`, `BlacklistedAlbumItem`, `DismissedArtistItem`, `BlacklistState`, `BlacklistActions` | 97 |
| `state/myqbz_state.slint` | `MixtapeCardItem`, `MyQbzState`, `MyQbzActions` | 83 |
| `state/myqbz_detail.slint` | `MixtapeDetailItem`, `MyQbzDetailState` | 118 |
| `state/myqbz_detail_actions_mix.slint` | `MyQbzDetailActions`, `MyQbzMixState`, `MyQbzMixActions` | 100 |
| `state/myqbz_edit_folder_edit.slint` | `MyQbzEditState`, `MyQbzEditActions`, `PmIconPreset`, `PmColorSwatch`, `FolderEditState`, `FolderEditActions` | 72 |
| `state/settings_state.slint` | `SettingsState` (single global — **132 lines, borderline**) | 132 |
| `state/appearance_state.slint` | `AppearanceState` (single global — **207 lines, exceeds by far**) | 207 |
| `state/branding_nav.slint` | `MyQbzBrandingState`, `NavMenuEntry`, `HeaderMenuState`, `NavState` | 87 |
| `state/artist_state.slint` | `LabelEntry`, `SimilarEntry`, `StoryItem`, `ArtistState` | 105 |
| `state/artist_actions_mb.slint` | `ArtworkActions`, `ArtistActions`, `MbOriginData`, `MbRelationship`, `MbRelationshipsData`, `DiscoveryArtist` | 98 |
| `state/network_musician.slint` | `NetworkSidebarState`, `NetworkSidebarActions`, `MusicianAppearanceItem`, `MusicianState`, `MusicianActions` | 103 |
| `state/label_artist_releases.slint` | `LabelState`, `LabelActions`, `ArtistReleasesState`, `ArtistReleasesActions` | 122 |
| `state/shell_state.slint` | `LocationViewState`, `LocationViewActions`, `UiScale`, `ShellState` | 125 |
| `state/session_queue_visualizer.slint` | `SessionState`, `QueueItem`, `VisualizerState` | 34 |
| `state/immersive_window_control.slint` | `ImmersiveState`, `WindowControlActions` | 105 |
| `state/now_playing_state.slint` | `NowPlayingState` (single global, 121) | 121 |
| `state/tooltip_artpreview.slint` | `TooltipState`, `ArtPreviewState` | 25 |
| `state/queue_state.slint` | `QueueState` (single global, 119) | 119 |
| `state/sleep_log_diag.slint` | `SleepTimerState`, `SleepTimerActions`, `LogRow`, `LogViewerState`, `ReportIssueState`, `ReportIssueActions`, `DiagRow`, `DiagnosticsState` | 76 |
| `state/playlist_picker_duplicate.slint` | `PlaylistPickItem`, `PlaylistPickerState`, `PlaylistPickerActions`, `DuplicateConfirmState`, `DuplicateConfirmActions` | 83 |
| `state/favorites_structs.slint` | `FavoriteArtistItem`, `FavArtistSection`, `FavoriteLabelItem` | 31 |
| `state/favorites_state_actions.slint` | `FavoritesState`, `FavoritesActions` | 112 |
| `state/library_all_mix.slint` | `LibraryFeedItem`, `LibraryAllState`, `LibraryAllActions`, `MixState` | 83 |
| `state/search_state.slint` | `SearchState`, `SearchActions` | 120 |
| `state/offline_login.slint` | `OfflineState`, `LoginState`, `OfflineModeActions` | 68 |
| `state/offline_favorites_playlist_import.slint` | `OfflineFavoritesState`, `OfflineFavoritesActions`, `ImportLogEntry`, `PlaylistImportState`, `PlaylistImportActions` | 91 |
| `state/dac_wizard.slint` | `RemediationRow`, `DacCandidateRow`, `DacConfigRow`, `DacWizardState`, `DacWizardActions` | 99 |
| `state/sandbox_keybindings.slint` | `SandboxState`, `KeybindingRow`, `KeybindingCategoryGroup`, `KeybindingsState`, `KeybindingsActions`, `KeyboardShortcutsState` | 70 |
| `state/link_resolver_about.slint` | `LinkResolverState`, `LinkResolverActions`, `AboutContributorRow`, `AboutContributorGroup`, `AboutState`, `AboutActions` | 77 |
| `state/whats_new.slint` | `WhatsNewBlock`, `WhatsNewTocEntry`, `WhatsNewState`, `WhatsNewActions` | 31 |

That is 48 new files. Six exceed 130 lines because they are (or contain)
a single atomic `global`/`struct` block: `local_library_state.slint` (146),
`appearance_state.slint` (207 — by far the worst), `settings_state.slint`
(132), plus three that land at 132-146 from an unsplittable pair. These
cannot be mechanically split further without either (a) breaking one
`global` into two cooperating globals (a real behavior-preserving
refactor of the Slint code, e.g. splitting `AppearanceState`'s ~180
properties into `AppearanceState` + `AppearanceColorsState`, updating
every `AppearanceState.foo` call site to `AppearanceColorsState.foo` for
the moved properties), or (b) accepting the exception. Flag these for a
deliberate follow-up decision rather than a silent oversized file.

## 3. Re-export / public API surface

`crates/qbz-ui/ui/state.slint` itself becomes the barrel:

```slint
export { AlbumCardItem, AlphaJump, ... } from "state/cards_misc.slint";
export { PinnedState, PinnedActions, ... } from "state/pinned_home.slint";
... (one export line per new file, every symbol re-listed)
```

Every existing `import { X, Y } from "../state.slint"` (or `"state.slint"`,
`"./state.slint"`) across ~150+ `.slint` files keeps working unchanged,
since `state.slint` still exports every symbol by the same name.

## 4. Tricky coupling to watch out for

- **Cross-global references inside the same block**: several
  globals/structs reference each other's types in-line (e.g. `SidebarRow`
  in Sidebar.slint uses `SidebarEntry`, `DragState`, `HeaderMenuState`,
  `NavMenuEntry`, `SidebarFolderPopupState`, `SidebarTooltipState`,
  `MyQbzBrandingState`, `OfflineState`, `NavState` — all from *different*
  proposed files). Since Slint globals reference each other by name and
  everything is re-exported transitively through `state.slint`, this is
  fine for external consumers, but the **new files under `state/` must
  import each other directly** wherever one global's property type is
  another global/struct (e.g. `state/sidebar_structs_popups.slint`
  referencing `NavMenuEntry` needs `import { NavMenuEntry } from
  "./branding_nav.slint";`). Map every cross-reference before moving code,
  not after — a missed import shows up only as a compile error deep in
  the split.
- **Struct-before-global ordering**: many structs are used as field types
  inside globals defined right after them in the current file (e.g.
  `PinnedItem` struct immediately before `PinnedState` global that uses
  it as `[PinnedItem]`). Keep such tightly-coupled struct+global pairs in
  the same new file (already done above) to avoid an extra import for
  every trivial pairing.
- **`AppearanceState`/`SettingsState`/`LocalLibraryState` are single
  atomic globals** — you cannot split one `global X { ... }` block
  across two files in Slint. If these must come under 130 lines, it
  requires an actual code change (split into two cooperating globals)
  and updating every call site — bigger than the other files' plans.
  Decide up front whether "exception, documented" is acceptable for this
  branch's goal, or whether these three get a dedicated deeper pass.
- **Import path depth**: new files live one level deeper
  (`ui/state/*.slint` vs `ui/state.slint`), so any `@image-url(...)` or
  relative import *inside* the moved global/struct bodies (there
  shouldn't normally be any inside `global`/`struct` blocks, but double
  check) needs a `../` prefix adjustment.

## 5. What to verify after the real split

- `cargo build -p qbz-ui` (and whichever crate embeds the Slint sources)
  compiles clean — Slint compile errors surface at build time, not as a
  separate lint step.
- Grep the whole repo for `from "state.slint"` / `from "../state.slint"`
  / `from "./state.slint"` and confirm the import list still resolves
  (no symbol renamed/dropped) — a diff of `grep -oh '{[^}]*}' state.slint`
  before/after should show the same symbol set.
- Run the app / `cargo test` across `qbz-ui`, `qbz-app`, and `qbz` (the
  Slint state feeds many Rust controllers) to catch any Rust-side
  `slint::include_modules!`-generated binding that assumed a specific
  module path.
- Spot-check a few UI surfaces that reference many globals at once
  (Sidebar, Settings, NowPlaying) actually render, since these are the
  files most likely to need the cross-file imports noted in §4.
