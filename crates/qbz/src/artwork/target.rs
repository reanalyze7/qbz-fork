//! `ArtworkTarget` — one variant per card/row slot the whole UI can request
//! artwork for, plus its per-target decode size.
//!
//! DELIBERATE EXCEPTION to the 130-line budget: this is a pure enum (no
//! I/O, no branching logic beyond `decode_size`'s straightforward match) —
//! every variant needs to stay visible together for `apply::apply_artwork`'s
//! dispatch and for reviewers scanning "what can request artwork". Splitting
//! it across files by UI area would force every future addition to guess
//! which file a new variant belongs in, and would scatter the one enum any
//! artwork call site pattern-matches on. Flagged for reviewer sign-off per
//! the refactor plan rather than silently splitting a data enum.

use crate::artwork::jobs::{scaled_decode, DECODE_SIZE};

/// Which card an artwork download targets.
/// (`Clone` only — `LocalAlbumById` carries a `String`, so no `Copy`.)
#[derive(Clone)]
pub enum ArtworkTarget {
    /// A card in a Discover descriptor list's embedded album section
    /// (`DiscoverState.home-sections` / `editor-sections`
    /// `[section_idx].section.albums[album_idx]`) — Slice 5's prefs-driven
    /// Home/Editor render loop. `editor` picks which list the job targets.
    DiscoverSectionAlbum {
        editor: bool,
        section_idx: usize,
        album_idx: usize,
    },
    /// A card in `HomeState.popular[idx]`.
    Popular { idx: usize },
    /// A card in `HomeState.recent[idx]`.
    Recent { idx: usize },
    /// A card in `HomeState.recent-albums[idx]`.
    RecentAlbum { idx: usize },
    /// A card in `RecentAlbumsState.albums[idx]` — the full "Recently Played
    /// Albums" page (the Home rail's "View all"). Own target because the page
    /// model has its own lifecycle, separate from the rail's (same split as
    /// HomeFavoriteAlbum vs ForYouFavoriteAlbum).
    RecentAlbumsPage { idx: usize },
    /// A card in `MostPlayedAlbumsState.albums[idx]` — the "Most Played Albums"
    /// View-all page.
    MostPlayedAlbumsPage { idx: usize },
    /// A card in `HomeState.favorite-albums.albums[idx]` — the Home tab's
    /// "Library Albums" rail (#566). Separate from `ForYouFavoriteAlbum`:
    /// the two rails share the data pipeline but not the model lifecycle.
    HomeFavoriteAlbum { idx: usize },
    /// A card in `HomeState.most-played-albums.albums[idx]` — the Home tab's
    /// "Most Played Albums" rail (local play-count ranking).
    HomeMostPlayedAlbum { idx: usize },
    /// A card in `HomeState.release-watch.albums[idx]` — the Home tab's
    /// "Release Watch" rail (#566; ForYouReleaseWatch's Home twin).
    HomeReleaseWatchAlbum { idx: usize },
    /// A tile in `HomeState.top-artists[idx]` — the Home tab's "Your Top
    /// Artists" rail (#566; ForYouTopArtist's Home twin).
    HomeTopArtist { idx: usize },
    /// A single playlist cover of `HomeState.playlists[idx]` (single cover →
    /// slot 0, unlike the 4-slot SearchPlaylistCover/FavPlaylistCover).
    HomePlaylistCover { idx: usize },
    /// A single playlist cover of `PlaylistBrowseState.playlists[idx]` — the
    /// Qobuz Playlists "View all" page. `visible` shares the same model
    /// while no search is active, so the rendered grid updates too (same
    /// contract as DiscoverBrowseAlbum).
    PlaylistBrowseCover { idx: usize },
    /// A row in `SearchState.albums[idx]`.
    SearchAlbum { idx: usize },
    /// A row in `SearchState.tracks[idx]`.
    SearchTrack { idx: usize },
    /// A row in `SearchState.artists[idx]`.
    SearchArtist { idx: usize },
    /// One collage cover slot (0-3) of `SearchState.playlists[idx]`.
    SearchPlaylistCover { idx: usize, slot: usize },
    /// One micro-collage cover slot (0-3) of `SidebarState.entries[idx]`.
    SidebarPlaylistCover { idx: usize, slot: usize },
    /// The most-popular search hero (its kind is read from SearchState).
    SearchMostPopular,
    /// A cortinilla row addressed by its stable flat index: 0 = the
    /// `SearchState.top-result`, 1.. = the section rows in declaration order
    /// (the same flat-index convention the click/keyboard path uses).
    CortinillaRow { flat_index: usize },
    /// An immersive-search dropdown row in `ImmersiveState.search-sections`,
    /// addressed by its stable flat index (1..). The immersive cortinilla has
    /// NO top result (top = None), so flat-index 0 is never produced; every job
    /// targets a section row. Mirrors `CortinillaRow` but writes the immersive
    /// global instead of `SearchState`.
    ImmersiveSearchRow { flat_index: usize },
    /// A blocked-album row cover in `BlacklistState.album-items[idx]` (the
    /// Blacklist Manager Albums tab).
    BlacklistAlbum { idx: usize },
    /// A release card in `ArtistState.release-sections[section_idx]
    /// .albums[album_idx]`.
    ArtistRelease { section_idx: usize, album_idx: usize },
    /// The single "Novedad más reciente" highlight in `ArtistState.last-release`.
    ArtistLastRelease,
    /// A card in `ArtistReleasesState.albums[index]` (dedicated discography page).
    ArtistReleasesAlbum { index: usize },
    /// A Magazine story thumbnail in `ArtistState.stories[index]`.
    ArtistStory { index: usize },
    /// A row in `ArtistState.top-tracks[index]` (artist "Popular Tracks"
    /// list). Mirrors `LabelTopTrack`: Slint can't fetch network images, so
    /// the row's album-cover thumbnail only paints once this job decodes the
    /// `artwork_url` bytes into the row's `artwork` field (#631).
    ArtistTopTrack { index: usize },
    /// A curated playlist card in `ArtistState.playlists[index]` (the
    /// main-column Playlists carousel). Single cover (slot 0), like
    /// `LabelPlaylistCover`.
    ArtistPlaylistCover { index: usize },
    /// A card in the Library "All" mixed feed (`LibraryAllState.items-visible[index]`).
    /// Dispatched against the VISIBLE model, re-dispatched on each derive.
    LibraryAllCover { index: usize },
    /// A row in `ArtistState.library-tracks[index]` — the ArtistPage
    /// "In library" track list (library_by_artist seed). Row thumbnail.
    ArtistLibraryTrack { index: usize },
    /// A card in `ArtistState.library-albums[index]` — the ArtistPage
    /// "In library" album grid.
    ArtistLibraryAlbum { index: usize },
    /// A card in MusicianState.appearances[index].
    MusicianAppearance { index: usize },
    /// A card in LabelState.albums[index] (releases sub-view grid).
    LabelAlbum { index: usize },
    /// A row in LabelState.top-tracks[index] (landing).
    LabelTopTrack { index: usize },
    /// A card in LabelState.releases-section.albums[index] (landing).
    LabelReleaseAlbum { index: usize },
    /// A card in LabelState.library-albums[index] (landing, "In library" tab).
    LabelLibraryAlbum { index: usize },
    /// A row in LabelState.library-tracks[index] (landing, "In library" tab).
    LabelLibraryTrack { index: usize },
    /// A card in LabelState.critics-section.albums[index] (landing).
    LabelCriticsAlbum { index: usize },
    /// The cover of LabelState.playlists[index] (landing).
    LabelPlaylistCover { index: usize },
    /// A card in LabelState.artists[index] (landing).
    LabelArtist { index: usize },
    /// A card in LabelState.more-labels[index] (landing).
    LabelMoreLabel { index: usize },
    /// A card in AlbumState.more-from-artist.albums[index] (album-view
    /// "From the same artist" carousel).
    AlbumMoreFromArtist { index: usize },
    /// A card in AlbumState.suggestions-section.albums[index] (album-view
    /// "Listening suggestions" carousel).
    AlbumSuggestion { index: usize },
    /// A card in AlbumState.lastfm-suggestions-section.albums[index]
    /// (album-view Last.fm similar-albums carousel, under the suggestions).
    AlbumLastfmSuggestion { index: usize },
    /// A card in LocationViewState.artists[index].
    LocationArtist { index: usize },
    /// A row in FavoritesState.tracks[index].
    FavoriteTrack { index: usize },
    /// A Favorites album cover, addressed BY ID (windowed dispatch over
    /// `albums-visible` — id-keyed delivery is immune to derive re-sorts
    /// between dispatch and apply). `gen` is the favorites-albums generation
    /// at fetch time; a stale cover (the model was replaced by a reload) is
    /// dropped.
    FavoriteAlbumById { id: String, gen: u64 },
    /// A card in DiscoverBrowseState.albums[index].
    DiscoverBrowseAlbum { index: usize },
    /// A Local Library album cover, addressed BY ID (windowed dispatch over
    /// `albums-visible` — id-keyed delivery is immune to derive re-sorts
    /// between dispatch and apply). `gen` is the albums generation at fetch
    /// time; a stale cover (the model was replaced by a reload) is dropped.
    LocalAlbumById { id: String, gen: u64 },
    /// A card in LocalLibraryState.folders[index] (Folders-flat grid).
    LocalFolderCard { index: usize },
    /// A subfolder cover card in LocalLibraryState.folder-detail-subfolders[index]
    /// (Folders-tree detail pane).
    LocalFolderDetailCard { index: usize },
    /// A card in LocalLibraryState.artists-selected-albums[index] (the
    /// Artists tab right pane — the selected artist's albums).
    LocalArtistAlbumCard { index: usize },
    /// A rail-row avatar in LocalLibraryState.artists (Artists tab). Addressed
    /// by its index in the FLAT master; the apply arm resolves index -> name
    /// and routes through the name-keyed dual-setter (grouped sections are
    /// re-derived, so they must be matched by name). `gen` drops a stale paint
    /// after a reload/rescan.
    LocalArtistRowImage { index: usize, gen: u64 },
    /// The cover of the dedicated Local Library album view (LocalAlbumState).
    LocalAlbumViewCover,
    /// A card in FavoritesState.artists[index].
    FavoriteArtist { index: usize },
    /// A card in FavoritesState.labels[index].
    FavoriteLabel { index: usize },
    /// One collage cover slot (0-3) of a favorites playlist card. `following`
    /// picks the sub-tab source model (Following vs Library/favorites).
    FavPlaylistCover { following: bool, index: usize, slot: usize },
    /// An album card in the favorites Artists sidepanel — section `section`
    /// of `FavoritesState.selected-artist-sections`, album `index`.
    FavoriteArtistAlbum { section: usize, index: usize },
    /// A card in ForYouState.release-watch.albums[index].
    ForYouReleaseWatch { index: usize },
    /// A card in ForYouState.recent-albums.albums[index].
    ForYouRecentAlbum { index: usize },
    /// A row in ForYouState.recent-tracks[index].
    ForYouRecentTrack { index: usize },
    /// A tile in ForYouState.top-artists[index].
    ForYouTopArtist { index: usize },
    /// A tile in ForYouState.artists-to-follow[index].
    ForYouToFollow { index: usize },
    /// A card in ForYouState.more-from-library.albums[index].
    ForYouMoreFromLibrary { index: usize },
    /// A card in ForYouState.rediscover.albums[index].
    ForYouRediscover { index: usize },
    /// 4th-tab "Recommendations" rows (external-reco engine).
    ExtRecoRecArtistCommon { index: usize },
    ExtRecoRecArtistRecent { index: usize },
    ExtRecoTopArtist { index: usize },
    ExtRecoRecAlbum { index: usize },
    ExtRecoFreshAlbum { index: usize },
    ExtRecoDeepAlbum { index: usize },
    ExtRecoTopAlbum { index: usize },
    ExtRecoWeeklyExploration { index: usize },
    ExtRecoWeeklyJams { index: usize },
    /// A card in ForYouState.favorite-albums.albums[index].
    ForYouFavoriteAlbum { index: usize },
    /// A card in ForYouState.most-played-albums.albums[index].
    ForYouMostPlayedAlbum { index: usize },
    /// The Spotlight artist portrait.
    ForYouSpotlightArtist,
    /// A card in ForYouState.spotlight-albums[index].
    ForYouSpotlightAlbum { index: usize },
    /// A row in MixState.tracks[index].
    MixTrack { index: usize },
    /// A row in PlaylistState.tracks[index].
    PlaylistTrack { index: usize },
    /// The PlaylistState header cover.
    PlaylistCover,
    /// One collage cover slot (0-3) of
    /// `PlaylistManagerState.playlists[index]`.
    PmPlaylistCover { index: usize, slot: usize },
    /// One collage cover slot (0-3) of a tree row's playlist
    /// (`PlaylistManagerState.tree[index].playlist`).
    PmTreeCover { index: usize, slot: usize },
    /// One mosaic cover slot (0-8) of a My QBZ Mixtapes-grid card
    /// (`MyQbzState.mixtapes[index]`). Up to 9 slots (3x3 Collections);
    /// mixtapes use only 0-3.
    MyQbzMixtapeCover { index: usize, slot: usize },
    /// One mosaic cover slot (0-8) of a My QBZ Collections-grid card
    /// (`MyQbzState.collections[index]`).
    MyQbzCollectionCover { index: usize, slot: usize },
    /// A row thumbnail in the My QBZ collection-detail item list
    /// (`MyQbzDetailState.items[index]`). Matched by item position on apply so
    /// a later sort/filter keeps the cover (the rendered model is re-derived).
    MyQbzDetailRow { position: i32 },
    /// One hero-mosaic cover slot (0-8) of the My QBZ collection-detail view
    /// (`MyQbzDetailState.cover{N}`).
    MyQbzDetailCover { slot: usize },
    /// One collage cover slot (0-3) of an immersive Suggestions card
    /// (`SuggestionsState.cards[card_idx].cover{slot}`). Playlist cards use
    /// up to 3 slots (book collage), the radio card up to 4 (diamond collage).
    SuggestionCardCover { card_idx: usize, slot: usize },
    /// A row thumbnail in the immersive Suggestions recommended-tracks list
    /// (`SuggestionsState.tracks[idx]`).
    SuggestionTrackCover { idx: usize },
    /// A row thumbnail in the playlist "Suggested Songs" section
    /// (`PlaylistSuggestionsState.rows[idx]`). 40px row art — decode small.
    PlaylistSuggestionCover { idx: usize },
    /// A card in `PinnedState.items[idx]` — the mixed Pinned carousel (Home
    /// and For You share the ONE model). Kinds are mixed; the apply arm reads
    /// the row's `kind` to pick the field to write (album / artist `artwork`
    /// vs playlist `cover1` + dominant colour). Index-keyed, so jobs are only
    /// ever dispatched by `pinned_section::rebuild_pinned` right after it
    /// replaced the model — never from a stale row set.
    PinnedCard { idx: usize },
}

impl ArtworkTarget {
    /// Pixel size to decode the cover to. List-row thumbnails are tiny
    /// (~40px), so decoding them to the card size (264) would retain
    /// huge buffers in the model — a 2000-row playlist would hold
    /// hundreds of MB. Decode row thumbnails small.
    pub(in crate::artwork) fn decode_size(&self) -> u32 {
        scaled_decode(match self {
            ArtworkTarget::SearchTrack { .. }
            | ArtworkTarget::FavoriteTrack { .. }
            | ArtworkTarget::MixTrack { .. }
            | ArtworkTarget::PlaylistTrack { .. }
            | ArtworkTarget::SuggestionTrackCover { .. }
            | ArtworkTarget::PlaylistSuggestionCover { .. }
            // Label/Artist "Popular Tracks" rows are the same list-row
            // thumbnail as the track targets above (~40px rendered).
            | ArtworkTarget::LabelTopTrack { .. }
            | ArtworkTarget::LabelLibraryTrack { .. }
            | ArtworkTarget::ArtistTopTrack { .. }
            | ArtworkTarget::ArtistLibraryTrack { .. }
            | ArtworkTarget::LocalArtistRowImage { .. } => 96,
            // Sidebar micro-collage tiles render at ~10-20px; decode tiny.
            ArtworkTarget::SidebarPlaylistCover { .. } => 48,
            // Playlist Manager collage tiles render at ~70-140px.
            ArtworkTarget::PmPlaylistCover { .. } | ArtworkTarget::PmTreeCover { .. } => 160,
            // My QBZ mosaic tiles render at ~60-92px (184/2 or 184/3 grid).
            ArtworkTarget::MyQbzMixtapeCover { .. }
            | ArtworkTarget::MyQbzCollectionCover { .. }
            // Hero mosaic tiles render at ~62-93px (186/2 or 186/3 grid).
            | ArtworkTarget::MyQbzDetailCover { .. } => 160,
            // Detail list-row thumbnails render at 36px.
            ArtworkTarget::MyQbzDetailRow { .. } => 96,
            _ => DECODE_SIZE,
        })
    }
}
