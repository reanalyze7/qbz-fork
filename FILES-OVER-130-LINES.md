# Inventaire — fichiers dépassant la règle des 130 lignes (20/08/2026)

Généré le 20/08/2026 sur l'état actuel de `main`. Aucun fichier touché,
document de référence uniquement — c'est la liste brute qui manquait, déjà
évoquée en conversation mais jamais écrite noir sur blanc avant ce commit.

**485 fichiers au total** dépassent 130 lignes : **134 fichiers Slint** et
**351 fichiers Rust**. C'est la quasi-totalité du projet, pas une exception —
dette du projet upstream, pas introduite par ce fork.

## Ce qui va déjà disparaître sans qu'on ait à les découper

**71 des 485 fichiers** (colonne "Note" ci-dessous) sont marqués
`partira avec REMOVAL-SPEC.md` — ils appartiennent à une fonctionnalité déjà
décidée supprimée (Lyrics, Cast, Qobuz Connect, Kiosk, Immersive, Miniplayer,
Plex, Radio, Purchases, Awards). Une fois `REMOVAL-SPEC.md` exécuté, la
liste réelle à traiter tombe à ~414 fichiers, sans qu'on ait dépensé le
moindre effort de découpage dessus.

`crates/qbz/src/main.rs` (22 760L) a son propre document séparé :
`MAIN-RS-SPLIT-PLAN.md` — plan écrit, pas exécuté, zone à risque élevé
(une seule fonction `main()` de ~14 500 lignes, closures qui capturent du
contexte local).

Tout le reste — les ~340 fichiers ni marqués REMOVAL-SPEC ni `main.rs` — est
une dette non traitée à ce jour, aucun plan de découpage écrit pour eux
individuellement. Décision prise le 20/08/2026 : on ne s'attaque pas à ce
reste tant qu'une machine capable de compiler n'a pas validé les chantiers
déjà en cours.
## Fichiers Slint > 130 lignes (134)

| Lignes | Fichier | Note |
|---|---|---|
| 6084 | `crates/qbz-ui/ui/state.slint` |  |
| 2660 | `crates/qbz-ui/ui/locallibrary/LocalLibraryView.slint` |  |
| 2010 | `crates/qbz-ui/ui/favorites/FavoritesView.slint` |  |
| 1879 | `crates/qbz-ui/ui/artist/ArtistPageView.slint` |  |
| 1706 | `crates/qbz-ui/ui/immersive/ImmersiveView.slint` | partira avec REMOVAL-SPEC.md |
| 1549 | `crates/qbz-ui/ui/purchases/PurchasesView.slint` | partira avec REMOVAL-SPEC.md |
| 1527 | `crates/qbz-ui/ui/playlist/PlaylistView.slint` |  |
| 1483 | `crates/qbz-ui/ui/playlist/PlaylistManagerView.slint` |  |
| 1265 | `crates/qbz-ui/ui/myqbz/MixtapeDetailView.slint` |  |
| 1259 | `crates/qbz-ui/ui/album/AlbumPageView.slint` |  |
| 1258 | `crates/qbz-ui/ui/shell/HeaderBar.slint` |  |
| 1150 | `crates/qbz-ui/ui/shell/Sidebar.slint` |  |
| 1114 | `crates/qbz-ui/ui/settings/BlacklistManagerView.slint` |  |
| 1085 | `crates/qbz-ui/ui/shell/QueueSidebar.slint` |  |
| 1029 | `crates/qbz-ui/ui/shell/AppShell.slint` |  |
| 1025 | `crates/qbz-ui/ui/settings/AppearanceSettings.slint` |  |
| 861 | `crates/qbz-ui/ui/primitives/DacWizardModal.slint` |  |
| 829 | `crates/qbz-ui/ui/settings/LocalLibrarySettings.slint` |  |
| 806 | `crates/qbz-ui/ui/search/SearchResultsView.slint` |  |
| 787 | `crates/qbz-ui/ui/myqbz/DiscographyBuilderView.slint` | partira avec REMOVAL-SPEC.md |
| 774 | `crates/qbz-ui/ui/purchases/PurchaseDetailView.slint` | partira avec REMOVAL-SPEC.md |
| 769 | `crates/qbz-ui/ui/primitives/TrackRow.slint` |  |
| 696 | `crates/qbz-ui/ui/shell/SongCard.slint` |  |
| 675 | `crates/qbz-ui/ui/label/LabelPageView.slint` |  |
| 658 | `crates/qbz-ui/ui/discover/HomeView.slint` |  |
| 617 | `crates/qbz-ui/ui/album/LocalAlbumView.slint` |  |
| 576 | `crates/qbz-ui/ui/offline/OfflineManagerView.slint` |  |
| 562 | `crates/qbz-ui/ui/primitives/PlaylistImportModal.slint` |  |
| 554 | `crates/qbz-ui/ui/immersive/ImmersiveQueuePanel.slint` | partira avec REMOVAL-SPEC.md |
| 553 | `crates/qbz-ui/ui/myqbz/AddToMixtapeModal.slint` |  |
| 540 | `crates/qbz-ui/ui/shell/KioskShell.slint` | partira avec REMOVAL-SPEC.md |
| 539 | `crates/qbz-ui/ui/primitives/TagEditorModal.slint` |  |
| 531 | `crates/qbz-ui/ui/shell/AboutModal.slint` |  |
| 522 | `crates/qbz-ui/ui/discover/AlbumCard.slint` |  |
| 516 | `crates/qbz-ui/ui/shell/LogViewerModal.slint` |  |
| 492 | `crates/qbz-ui/ui/immersive/ImmersiveSuggestionsPanel.slint` | partira avec REMOVAL-SPEC.md |
| 483 | `crates/qbz-ui/ui/search/Cortinilla.slint` |  |
| 476 | `crates/qbz-ui/ui/album/AlbumCreditsModal.slint` |  |
| 446 | `crates/qbz-ui/ui/shell/KioskSearch.slint` | partira avec REMOVAL-SPEC.md |
| 440 | `crates/qbz-ui/ui/primitives/AlbumListRow.slint` |  |
| 436 | `crates/qbz-ui/ui/shell/LyricsLinesView.slint` | partira avec REMOVAL-SPEC.md |
| 402 | `crates/qbz-ui/ui/settings/LibFolderEditModal.slint` |  |
| 396 | `crates/qbz-ui/ui/miniplayer/MiniFooter.slint` | partira avec REMOVAL-SPEC.md |
| 390 | `crates/qbz-ui/ui/settings/DiagnosticsPanel.slint` |  |
| 388 | `crates/qbz-ui/ui/discover/GenreFilterPopup.slint` |  |
| 386 | `crates/qbz-ui/ui/immersive/ImmersiveSearchCortinilla.slint` | partira avec REMOVAL-SPEC.md |
| 383 | `crates/qbz-ui/ui/settings/AudioSettings.slint` |  |
| 374 | `crates/qbz-ui/ui/shell/KioskNowPlaying.slint` | partira avec REMOVAL-SPEC.md |
| 370 | `crates/qbz-ui/ui/discover/DiscoverConfigModal.slint` |  |
| 366 | `crates/qbz-ui/ui/shell/CustomizeShortcutsModal.slint` |  |
| 364 | `crates/qbz-ui/ui/award/AwardAlbumsView.slint` | partira avec REMOVAL-SPEC.md |
| 364 | `crates/qbz-ui/ui/album/TrackInfoModal.slint` |  |
| 361 | `crates/qbz-ui/ui/shell/LyricsSidebar.slint` | partira avec REMOVAL-SPEC.md |
| 361 | `crates/qbz-ui/ui/shell/CastPicker.slint` | partira avec REMOVAL-SPEC.md |
| 346 | `crates/qbz-ui/ui/award/AwardView.slint` | partira avec REMOVAL-SPEC.md |
| 345 | `crates/qbz-ui/ui/discover/AlbumCollectionView.slint` |  |
| 338 | `crates/qbz-ui/ui/discover/PlaylistBrowseView.slint` |  |
| 336 | `crates/qbz-ui/ui/settings/IntegrationsSettings.slint` |  |
| 335 | `crates/qbz-ui/ui/label/LabelReleasesView.slint` |  |
| 328 | `crates/qbz-ui/ui/discover/ArtistGridCard.slint` |  |
| 326 | `crates/qbz-ui/ui/primitives/QbzSelect.slint` |  |
| 323 | `crates/qbz-ui/ui/discover/TrackCard.slint` |  |
| 322 | `crates/qbz-ui/ui/mix/MixView.slint` |  |
| 316 | `crates/qbz-ui/ui/discover/Carousel.slint` |  |
| 314 | `crates/qbz-ui/ui/settings/SettingsView.slint` |  |
| 309 | `crates/qbz-ui/ui/album/AlbumBookletModal.slint` |  |
| 305 | `crates/qbz-ui/ui/app.slint` |  |
| 296 | `crates/qbz-ui/ui/musician/MusicianPageView.slint` |  |
| 294 | `crates/qbz-ui/ui/login/LoginScreen.slint` |  |
| 291 | `crates/qbz-ui/ui/discover/PlaylistCard.slint` |  |
| 290 | `crates/qbz-ui/ui/shell/LinkResolverModal.slint` |  |
| 286 | `crates/qbz-ui/ui/primitives/JumpNavBar.slint` |  |
| 285 | `crates/qbz-ui/ui/immersive/CoverflowPanel.slint` | partira avec REMOVAL-SPEC.md |
| 282 | `crates/qbz-ui/ui/immersive/ImmersiveTrackInfoPanel.slint` | partira avec REMOVAL-SPEC.md |
| 277 | `crates/qbz-ui/ui/primitives/FolderEditModal.slint` |  |
| 275 | `crates/qbz-ui/ui/shell/SidebarPlaylistsPopup.slint` |  |
| 260 | `crates/qbz-ui/ui/shell/KeyboardShortcutsModal.slint` |  |
| 260 | `crates/qbz-ui/ui/foundation/theme.slint` |  |
| 257 | `crates/qbz-ui/ui/artist/ArtistReleasesView.slint` |  |
| 253 | `crates/qbz-ui/ui/shell/WhatsNewModal.slint` |  |
| 253 | `crates/qbz-ui/ui/shell/SidebarNowPlayingDock.slint` |  |
| 252 | `crates/qbz-ui/ui/shell/LyricsControlsFlyout.slint` | partira avec REMOVAL-SPEC.md |
| 251 | `crates/qbz-ui/ui/immersive/SpectrumPanel.slint` | partira avec REMOVAL-SPEC.md |
| 249 | `crates/qbz-ui/ui/primitives/SearchTrackHero.slint` |  |
| 249 | `crates/qbz-ui/ui/primitives/ColorPicker.slint` |  |
| 237 | `crates/qbz-ui/ui/myqbz/CollectionsView.slint` |  |
| 237 | `crates/qbz-ui/ui/discover/DiscoverBrowseView.slint` |  |
| 236 | `crates/qbz-ui/ui/shell/TransportControls.slint` |  |
| 228 | `crates/qbz-ui/ui/primitives/TrackPlayCell.slint` |  |
| 227 | `crates/qbz-ui/ui/settings/PlaybackSettings.slint` |  |
| 225 | `crates/qbz-ui/ui/discover/ArtistCarousel.slint` |  |
| 222 | `crates/qbz-ui/ui/myqbz/MixtapesView.slint` |  |
| 221 | `crates/qbz-ui/ui/discover/ForYouView.slint` |  |
| 220 | `crates/qbz-ui/ui/discover/SlimCarousel.slint` |  |
| 211 | `crates/qbz-ui/ui/primitives/TrackContextMenu.slint` |  |
| 211 | `crates/qbz-ui/ui/discover/ExternalRecoView.slint` |  |
| 199 | `crates/qbz-ui/ui/myqbz/CreateMyQbzModal.slint` |  |
| 198 | `crates/qbz-ui/ui/discover/MostPlayedAlbumsView.slint` |  |
| 194 | `crates/qbz-ui/ui/discover/Spotlight.slint` |  |
| 192 | `crates/qbz-ui/ui/settings/SettingsExportModal.slint` |  |
| 192 | `crates/qbz-ui/ui/primitives/EditPlaylistModal.slint` |  |
| 191 | `crates/qbz-ui/ui/shell/AudioStamp.slint` |  |
| 191 | `crates/qbz-ui/ui/settings/OfflineSettings.slint` |  |
| 188 | `crates/qbz-ui/ui/shell/KioskLibrary.slint` | partira avec REMOVAL-SPEC.md |
| 184 | `crates/qbz-ui/ui/myqbz/MyQbzMixModal.slint` |  |
| 184 | `crates/qbz-ui/ui/miniplayer/MiniWindowControls.slint` | partira avec REMOVAL-SPEC.md |
| 184 | `crates/qbz-ui/ui/discover/PlaylistTagFilter.slint` |  |
| 180 | `crates/qbz-ui/ui/discover/PlaylistCarousel.slint` |  |
| 178 | `crates/qbz-ui/ui/immersive/AlbumReactivePanel.slint` | partira avec REMOVAL-SPEC.md |
| 176 | `crates/qbz-ui/ui/settings/SandboxSettings.slint` |  |
| 175 | `crates/qbz-ui/ui/discover/PinnedCarousel.slint` |  |
| 173 | `crates/qbz-ui/ui/location/ArtistsByLocationView.slint` |  |
| 173 | `crates/qbz-ui/ui/discover/RecentAlbumsView.slint` |  |
| 173 | `crates/qbz-ui/ui/discover/BrowseHeaderTools.slint` |  |
| 170 | `crates/qbz-ui/ui/discover/HomeSkeleton.slint` |  |
| 168 | `crates/qbz-ui/ui/shell/NavRail.slint` |  |
| 167 | `crates/qbz-ui/ui/shell/KioskAlbum.slint` | partira avec REMOVAL-SPEC.md |
| 164 | `crates/qbz-ui/ui/primitives/ExpandableSearch.slint` |  |
| 163 | `crates/qbz-ui/ui/shell/HeaderMenuOverlay.slint` |  |
| 163 | `crates/qbz-ui/ui/primitives/AlbumContextMenu.slint` |  |
| 163 | `crates/qbz-ui/ui/myqbz/MyQbzShared.slint` |  |
| 154 | `crates/qbz-ui/ui/shell/SeekBar.slint` |  |
| 154 | `crates/qbz-ui/ui/immersive/ImmersiveLyricsFocusPanel.slint` | partira avec REMOVAL-SPEC.md |
| 153 | `crates/qbz-ui/ui/shell/KioskMyQBZ.slint` | partira avec REMOVAL-SPEC.md |
| 153 | `crates/qbz-ui/ui/primitives/PlaylistListRow.slint` |  |
| 152 | `crates/qbz-ui/ui/shell/QconnectDevModal.slint` | partira avec REMOVAL-SPEC.md |
| 147 | `crates/qbz-ui/ui/myqbz/MyQbzEditModal.slint` |  |
| 146 | `crates/qbz-ui/ui/shell/SidebarFolderPopup.slint` |  |
| 145 | `crates/qbz-ui/ui/discover/ArtistCard.slint` |  |
| 143 | `crates/qbz-ui/ui/immersive/ImmersiveTrackInfo.slint` | partira avec REMOVAL-SPEC.md |
| 142 | `crates/qbz-ui/ui/artist/ReleaseGrid.slint` |  |
| 141 | `crates/qbz-ui/ui/shell/ReportIssueModal.slint` |  |
| 139 | `crates/qbz-ui/ui/miniplayer/MiniQueueSurface.slint` | partira avec REMOVAL-SPEC.md |
| 132 | `crates/qbz-ui/ui/primitives/CircleAction.slint` |  |

## Fichiers Rust > 130 lignes (351)

| Lignes | Fichier | Note |
|---|---|---|
| 22760 | `crates/qbz/src/main.rs` | voir MAIN-RS-SPLIT-PLAN.md, plan écrit non exécuté |
| 6741 | `crates/qbz-library/src/database.rs` |  |
| 5617 | `crates/qbz-player/src/player/mod.rs` |  |
| 5395 | `crates/qbz/src/playback.rs` |  |
| 4211 | `crates/qbz/src/local_library.rs` |  |
| 3195 | `crates/qbz-core/src/core.rs` |  |
| 3184 | `crates/qbz/src/purchases.rs` | partira avec REMOVAL-SPEC.md |
| 3180 | `crates/qbz-qobuz/src/client.rs` |  |
| 3024 | `crates/qconnect-app/src/app.rs` |  |
| 2756 | `crates/qbz/src/qconnect_service.rs` | partira avec REMOVAL-SPEC.md |
| 2346 | `crates/qbz-player/src/queue.rs` |  |
| 2059 | `crates/qbz/src/local_playlist.rs` |  |
| 2057 | `crates/qbz-plex/src/lib.rs` | partira avec REMOVAL-SPEC.md |
| 1958 | `crates/qbz/src/search.rs` |  |
| 1927 | `crates/qbz/src/artist.rs` |  |
| 1921 | `crates/qbzd/src/tui/app.rs` |  |
| 1851 | `crates/qbz-theme/src/registry.rs` |  |
| 1832 | `crates/qbz/src/artwork.rs` |  |
| 1673 | `crates/qbz/src/favorites.rs` |  |
| 1619 | `crates/qbz/src/settings.rs` |  |
| 1606 | `crates/qbz/src/cast_service.rs` | partira avec REMOVAL-SPEC.md |
| 1569 | `crates/qbz-app/src/settings/bundle.rs` |  |
| 1535 | `crates/qbz-audio/src/alsa_backend.rs` |  |
| 1528 | `crates/qbz-models/src/types.rs` |  |
| 1415 | `crates/qbz/src/myqbz_detail.rs` |  |
| 1397 | `crates/qbzd/src/cli/settings.rs` |  |
| 1380 | `crates/qconnect-transport-ws/src/transport.rs` |  |
| 1368 | `crates/qconnect-protocol/src/decoder.rs` |  |
| 1339 | `crates/qbz-app/src/settings/reco_store.rs` |  |
| 1321 | `crates/qbz-library/src/metadata.rs` |  |
| 1311 | `crates/qconnect-protocol/src/mapper.rs` |  |
| 1286 | `crates/qconnect-app/src/renderer.rs` |  |
| 1275 | `crates/qbz-lyrics/src/service.rs` | partira avec REMOVAL-SPEC.md |
| 1269 | `crates/qbz-offline-cache/src/purchases_service.rs` | partira avec REMOVAL-SPEC.md |
| 1249 | `crates/qbz/src/queue.rs` |  |
| 1235 | `crates/qbz-credentials/src/lib.rs` |  |
| 1234 | `crates/qbz/src/diagnostics.rs` |  |
| 1226 | `crates/qbz/src/home.rs` |  |
| 1206 | `crates/qbz-player/src/player/streaming_source.rs` |  |
| 1204 | `crates/qbz/src/label.rs` |  |
| 1193 | `crates/qbzd/src/tui/screens/wizard.rs` |  |
| 1137 | `crates/qbz-reco/src/suggestions.rs` |  |
| 1136 | `crates/qbzd/src/daemon.rs` |  |
| 1112 | `crates/qbz/src/playlist.rs` |  |
| 1094 | `crates/qbzd/src/tui/screens/audio.rs` |  |
| 1082 | `crates/qbz/src/album.rs` |  |
| 1053 | `crates/qbz/src/foryou.rs` |  |
| 1052 | `crates/qbz-audio/src/pipewire_backend.rs` |  |
| 1043 | `crates/qbz/src/playlist_manager.rs` |  |
| 1036 | `crates/qbz/src/shader_underlay.rs` |  |
| 1032 | `crates/qbz-audio/src/settings.rs` |  |
| 1030 | `crates/qbz/src/plex_auth.rs` | partira avec REMOVAL-SPEC.md |
| 1029 | `crates/qconnect-protocol/src/queue_command_proto.rs` |  |
| 1008 | `crates/qbzd/src/tui/widgets.rs` |  |
| 983 | `crates/qbz/src/ui_prefs.rs` |  |
| 981 | `crates/qbz-player/src/player/playback_engine.rs` |  |
| 972 | `crates/qbz-audio/src/alsa_direct.rs` |  |
| 966 | `crates/qbz-integrations/src/listenbrainz/client.rs` |  |
| 956 | `crates/qbz/src/myqbz_builder.rs` |  |
| 945 | `crates/qbz/src/external_reco.rs` |  |
| 932 | `crates/qbz-audio/src/backend.rs` |  |
| 922 | `crates/qbz-integrations/src/lastfm/client.rs` | partira avec REMOVAL-SPEC.md |
| 893 | `crates/qbz-app/src/playback_driver.rs` |  |
| 880 | `crates/qbzd/src/login.rs` |  |
| 821 | `crates/qbz-library/src/local_playlists.rs` |  |
| 816 | `crates/qbz/src/local_library_settings.rs` |  |
| 809 | `crates/qbz-integrations/src/musicbrainz/client.rs` |  |
| 794 | `crates/qbz-dac-wizard/src/lib.rs` |  |
| 777 | `crates/qbz/src/scrobble.rs` |  |
| 775 | `crates/qbz-external-reco/src/carousels.rs` |  |
| 760 | `crates/qbz/src/myqbz_play.rs` |  |
| 760 | `crates/qbz-offline-cache/src/downloader.rs` |  |
| 760 | `crates/qbz-mixtape/src/shuffle.rs` |  |
| 755 | `crates/qbzd/src/api/queue.rs` |  |
| 753 | `crates/qbz-offline-cache/src/db.rs` |  |
| 750 | `crates/qbz/src/lyrics.rs` | partira avec REMOVAL-SPEC.md |
| 749 | `crates/qbz/src/sidebar.rs` |  |
| 749 | `crates/qbz-mixtape/src/enqueue.rs` |  |
| 745 | `crates/qbz-cast/src/dlna/device.rs` | partira avec REMOVAL-SPEC.md |
| 736 | `crates/qbz-mixtape/src/repo.rs` |  |
| 729 | `crates/qbz-app/src/settings/remote_control.rs` |  |
| 724 | `crates/qbz-integrations/src/musicbrainz/cache.rs` |  |
| 723 | `crates/qbz-app/src/settings/plex.rs` | partira avec REMOVAL-SPEC.md |
| 718 | `crates/qbz-app/src/settings/discover_prefs.rs` |  |
| 712 | `crates/qbz-cast/src/media_server.rs` | partira avec REMOVAL-SPEC.md |
| 705 | `crates/qbzd/src/qconnect/transport.rs` |  |
| 704 | `crates/qbz-audio/src/device_reservation/linux.rs` |  |
| 686 | `crates/qbzd/src/api/mod.rs` |  |
| 683 | `crates/qbz/src/playlist_suggestions.rs` |  |
| 673 | `crates/qbz-theme/src/auto/system.rs` |  |
| 667 | `crates/qbz-app/src/settings/artist_blacklist.rs` |  |
| 661 | `crates/qbz-app/src/session_store.rs` |  |
| 660 | `crates/qbz/src/miniplayer.rs` | partira avec REMOVAL-SPEC.md |
| 653 | `crates/qbz-integrations/src/musicbrainz/models.rs` |  |
| 649 | `crates/qbz/src/myqbz.rs` |  |
| 643 | `crates/qbz/src/keybindings.rs` |  |
| 642 | `crates/qbz/src/award.rs` | partira avec REMOVAL-SPEC.md |
| 630 | `crates/qbz/src/qconnect_event_sink.rs` | partira avec REMOVAL-SPEC.md |
| 629 | `crates/qbzd/src/tui/wizard_core.rs` |  |
| 626 | `crates/qbz-integrations/src/discogs/mod.rs` | partira avec REMOVAL-SPEC.md |
| 615 | `crates/qconnect-app/src/queue_resolution.rs` |  |
| 614 | `crates/qconnect-app/src/session.rs` |  |
| 611 | `crates/qbz/src/tag_editor.rs` |  |
| 601 | `crates/qbz/src/qconnect_transport.rs` | partira avec REMOVAL-SPEC.md |
| 598 | `crates/qbz-app/src/offline_mode/connectivity.rs` |  |
| 597 | `crates/qbz/src/offline_cache.rs` |  |
| 576 | `crates/qbz-app/src/settings/bundle/tests.rs` |  |
| 574 | `crates/qbzd/src/main.rs` |  |
| 573 | `crates/qbz-qobuz/src/lyrics.rs` | partira avec REMOVAL-SPEC.md |
| 563 | `crates/qbz/src/info_modals.rs` |  |
| 553 | `crates/qbz-playlist-import/src/match_qobuz.rs` |  |
| 553 | `crates/qbzd/src/tui/screens/playback.rs` |  |
| 546 | `crates/qbzd/src/api/playback.rs` |  |
| 541 | `crates/qbz-app/src/settings/favorites_cache.rs` |  |
| 538 | `crates/qbzd/src/qconnect/mod.rs` |  |
| 533 | `crates/qbz-qobuz/src/cmaf.rs` |  |
| 526 | `crates/qbz-reco/src/store.rs` |  |
| 518 | `crates/qbz-app/src/offline_mode/mod.rs` |  |
| 517 | `crates/qbzd/src/tui/screens/bundle.rs` |  |
| 515 | `crates/qbz/src/mix.rs` |  |
| 510 | `crates/qbz-radio/src/db.rs` | partira avec REMOVAL-SPEC.md |
| 509 | `crates/qbzd/src/cli/queue.rs` |  |
| 506 | `crates/qconnect-core/src/reducer.rs` |  |
| 500 | `crates/qbz-theme/src/auto/generator.rs` |  |
| 500 | `crates/qbz/src/library_all.rs` |  |
| 494 | `crates/qbz/src/playlist_import.rs` |  |
| 494 | `crates/qbz-models/src/source.rs` |  |
| 493 | `crates/qbzd/src/cli/transport.rs` |  |
| 488 | `crates/qbz-lyrics/src/wsync.rs` | partira avec REMOVAL-SPEC.md |
| 487 | `crates/qbz-app/src/runtime.rs` |  |
| 486 | `crates/qbz-app/src/settings/search_ranking.rs` |  |
| 485 | `crates/qbz/src/tray/linux.rs` |  |
| 485 | `crates/qbz-qobuz/src/purchases.rs` | partira avec REMOVAL-SPEC.md |
| 484 | `crates/qbz/src/genre_filter.rs` |  |
| 471 | `crates/qbz-audio/src/coreaudio_direct.rs` |  |
| 469 | `crates/qbz-playlist-import/src/providers/tidal.rs` |  |
| 468 | `crates/qbz-text-utils/src/strip_html.rs` |  |
| 465 | `crates/qbz/src/nav.rs` |  |
| 463 | `crates/qbz-dsd/src/demux.rs` |  |
| 462 | `crates/qbz-app/src/settings/scrobblers.rs` |  |
| 452 | `crates/qbz-app/src/settings/playback.rs` |  |
| 439 | `crates/qbzd/src/tui/strings.rs` |  |
| 438 | `crates/qbz/src/offline_manager.rs` |  |
| 435 | `crates/qbz-library/src/ephemeral.rs` |  |
| 428 | `crates/qbz/src/auth.rs` |  |
| 426 | `crates/qbz-app/src/offline_mode/store.rs` |  |
| 425 | `crates/qbz/src/suggestions.rs` |  |
| 425 | `crates/qbz/src/immersive.rs` | partira avec REMOVAL-SPEC.md |
| 422 | `crates/qbz/src/lyrics_sync.rs` | partira avec REMOVAL-SPEC.md |
| 417 | `crates/qbz-app/src/shell.rs` |  |
| 416 | `crates/qbz-lyrics/src/providers.rs` | partira avec REMOVAL-SPEC.md |
| 416 | `crates/qbz-app/src/settings/tray.rs` |  |
| 413 | `crates/qbz-qobuz/src/bundle.rs` |  |
| 412 | `crates/qbz/src/whats_new.rs` |  |
| 412 | `crates/qbz/src/theme.rs` |  |
| 412 | `crates/qbz-app/src/settings/search_cache.rs` |  |
| 410 | `crates/qbz-lyrics/src/lrc.rs` | partira avec REMOVAL-SPEC.md |
| 405 | `crates/qbz-reco/src/builder.rs` |  |
| 403 | `crates/qbz-qobuz/src/performers.rs` |  |
| 402 | `crates/qbz-app/src/settings/favorites.rs` |  |
| 395 | `crates/qbz/src/lyrics_prefs.rs` | partira avec REMOVAL-SPEC.md |
| 395 | `crates/qbz-reco/src/sparse_vector.rs` |  |
| 393 | `crates/qbzd/src/api/status.rs` |  |
| 392 | `crates/qbz-music-link/src/detection.rs` |  |
| 391 | `crates/qbz/src/tray/mod.rs` |  |
| 388 | `crates/qbz-library/src/scan.rs` |  |
| 388 | `crates/qbz-integrations/src/remote_metadata/mod.rs` | partira avec REMOVAL-SPEC.md |
| 386 | `crates/qbz-lyrics/src/model.rs` | partira avec REMOVAL-SPEC.md |
| 384 | `crates/qbz-core/src/system_capabilities.rs` |  |
| 383 | `crates/qbz/src/playlist_browse.rs` |  |
| 381 | `crates/qbz-lyrics/src/cache.rs` | partira avec REMOVAL-SPEC.md |
| 379 | `crates/qbz/src/myqbz_add.rs` |  |
| 378 | `crates/qbz/src/discover_prefs.rs` |  |
| 376 | `crates/qbz-audio/src/loudness.rs` |  |
| 372 | `crates/qbz/src/fav_cache.rs` |  |
| 367 | `crates/qbz/src/artist_blacklist.rs` |  |
| 367 | `crates/qbz-offline-cache/src/metadata.rs` |  |
| 361 | `crates/qbz-audio/src/visualizer/processor.rs` |  |
| 359 | `crates/qbz-cast/src/chromecast/device.rs` | partira avec REMOVAL-SPEC.md |
| 356 | `crates/qbz-app/src/diagnostics.rs` |  |
| 353 | `crates/qbz-external-reco/src/validate.rs` |  |
| 353 | `crates/qbz-audio/src/health.rs` |  |
| 352 | `crates/qbz/src/blacklist_manager.rs` |  |
| 352 | `crates/qbzd/src/qconnect/engine.rs` |  |
| 351 | `crates/qbz-theme/src/custom.rs` |  |
| 351 | `crates/qbz-media-controls/src/linux.rs` |  |
| 351 | `crates/qbzd/src/qconnect/session.rs` |  |
| 351 | `crates/qbz-cast/src/chromecast/thread.rs` | partira avec REMOVAL-SPEC.md |
| 346 | `crates/qbz-music-link/src/odesli.rs` |  |
| 344 | `crates/qbz/src/myqbz_edit.rs` |  |
| 341 | `crates/qbzd/src/qconnect/sink.rs` |  |
| 340 | `crates/qbzd/src/api/search.rs` |  |
| 338 | `crates/qbz-app/src/settings/graphics.rs` |  |
| 336 | `crates/qbz-models/src/purchase_serde.rs` | partira avec REMOVAL-SPEC.md |
| 336 | `crates/qbzd/src/tui/clipboard.rs` |  |
| 335 | `crates/qbzd/src/cli/service.rs` |  |
| 332 | `crates/qbzd/src/cli/status.rs` |  |
| 332 | `crates/qbz-app/src/settings/album_play_history.rs` |  |
| 331 | `crates/qbz/src/tray/macos.rs` |  |
| 330 | `crates/qbz/src/offline_mode.rs` |  |
| 326 | `crates/qbz/src/session_persist.rs` |  |
| 326 | `crates/qbz-playlist-import/src/providers/apple.rs` |  |
| 325 | `crates/qbzd/src/scrobble_engine.rs` |  |
| 323 | `crates/qbz-app/src/settings/pinned_items.rs` |  |
| 322 | `crates/qbzd/src/tui/screens/network.rs` |  |
| 322 | `crates/qbz-app/src/playback_context.rs` |  |
| 321 | `crates/qbzd/src/qconnect/remote_stream.rs` |  |
| 318 | `crates/qbz/src/remote_stream.rs` |  |
| 318 | `crates/qbz-cache/src/playback_cache.rs` |  |
| 317 | `crates/qbz/src/visualizer.rs` |  |
| 316 | `crates/qbz-media-controls/src/notify.rs` |  |
| 315 | `crates/qbz-integrations/src/musicbrainz/location.rs` |  |
| 315 | `crates/qbz-integrations/src/musicbrainz/genre.rs` |  |
| 314 | `crates/qbz-library/src/cue_parser.rs` |  |
| 311 | `crates/qbz-qobuz/src/link_resolver.rs` |  |
| 311 | `crates/qbz-external-reco/src/cache.rs` |  |
| 310 | `crates/qbz-cmaf/src/parser.rs` |  |
| 309 | `crates/qbz-theme/src/auto/palette.rs` |  |
| 307 | `crates/qbz-app/src/settings/local_favorites.rs` |  |
| 306 | `crates/qbz-library/src/qobuz_playlist_snapshot.rs` |  |
| 300 | `crates/qbz-i18n/src/po.rs` |  |
| 299 | `crates/qbz/src/reco.rs` |  |
| 299 | `crates/qbz-playlist-import/src/providers/spotify.rs` |  |
| 299 | `crates/qbz-app/src/settings/subscription.rs` |  |
| 297 | `crates/qbz-library/src/models.rs` |  |
| 296 | `crates/qbzd/src/cli/client.rs` |  |
| 296 | `crates/qbzd/src/api/play.rs` |  |
| 291 | `crates/qbz/src/lyrics_measure.rs` | partira avec REMOVAL-SPEC.md |
| 287 | `crates/qbz-theme/src/auto/mod.rs` |  |
| 287 | `crates/qbz-integrations/src/listenbrainz/cache.rs` |  |
| 287 | `crates/qbz-i18n/src/lib.rs` |  |
| 286 | `crates/qbz-theme/src/id.rs` |  |
| 285 | `crates/qbzd/src/qconnect/report.rs` |  |
| 282 | `crates/qbz-dsd/src/convert.rs` |  |
| 281 | `crates/qbz/src/recently.rs` |  |
| 277 | `crates/qbz/src/log_viewer.rs` |  |
| 276 | `crates/qbz/src/custom_theme.rs` |  |
| 275 | `crates/qbz-radio/src/builder.rs` | partira avec REMOVAL-SPEC.md |
| 273 | `crates/qbz-cache/src/audio_cache.rs` |  |
| 271 | `crates/qbzd/src/mpris.rs` |  |
| 269 | `crates/qbz-library/src/mount_info.rs` |  |
| 268 | `crates/qbz-integrations/src/listenbrainz/models.rs` |  |
| 266 | `crates/qbz-dsd/src/dop.rs` |  |
| 264 | `crates/qbz/src/album_map.rs` |  |
| 264 | `crates/qbz-audio/src/network_throttle.rs` |  |
| 261 | `crates/qbz-offline-cache/src/migration.rs` |  |
| 258 | `crates/qbzd/src/tui/screens/qconnect.rs` |  |
| 257 | `crates/qbz/src/myqbz_prefs.rs` |  |
| 256 | `crates/qbz-qobuz/src/auth.rs` |  |
| 256 | `crates/qbz-lyrics/src/sync.rs` | partira avec REMOVAL-SPEC.md |
| 256 | `crates/qbzd/src/cli/copy.rs` |  |
| 252 | `crates/qbz-audio/src/loudness_analyzer.rs` |  |
| 251 | `crates/qbz/src/qconnect_engine.rs` | partira avec REMOVAL-SPEC.md |
| 246 | `crates/qbz/src/offline_favorites.rs` |  |
| 245 | `crates/qbz/src/myqbz_mix.rs` |  |
| 244 | `crates/qbz/src/discover_browse.rs` |  |
| 242 | `crates/qbzd/src/api/browse.rs` |  |
| 241 | `crates/qbz-app/src/settings/search_service.rs` |  |
| 238 | `crates/qbz-offline-cache/src/cmaf_store.rs` |  |
| 237 | `crates/qbz-integrations/src/remote_metadata/models.rs` | partira avec REMOVAL-SPEC.md |
| 236 | `crates/qbz/src/folders.rs` |  |
| 236 | `crates/qbz/src/deep_link.rs` |  |
| 236 | `crates/qbz-offline-cache/src/path_validator.rs` |  |
| 234 | `crates/qbz/src/reco_dismiss.rs` |  |
| 232 | `crates/qbz/src/myqbz_cover.rs` |  |
| 231 | `crates/qbz-dsd/tests/demux_convert.rs` |  |
| 224 | `crates/qbz-audio/src/device_filter.rs` |  |
| 222 | `crates/qbz-external-reco/src/matching.rs` |  |
| 220 | `crates/qbz-qobuz/examples/lyrics_probe.rs` | partira avec REMOVAL-SPEC.md |
| 220 | `crates/qbz-playlist-import/src/providers/mod.rs` |  |
| 220 | `crates/qbzd/src/cli/browse.rs` |  |
| 219 | `crates/qconnect-core/src/renderer.rs` |  |
| 216 | `crates/qbzd/src/api/playlist.rs` |  |
| 213 | `crates/qbzd/src/cli/playlist.rs` |  |
| 213 | `crates/qbz-app/src/settings/developer.rs` |  |
| 210 | `crates/qbz/src/pinned.rs` |  |
| 210 | `crates/qbz-integrations/src/discord.rs` | partira avec REMOVAL-SPEC.md |
| 210 | `crates/qbzd/src/cli/search.rs` |  |
| 205 | `crates/qbz-cache/src/image_cache.rs` |  |
| 205 | `crates/qbz-audio/src/dac_probe.rs` |  |
| 205 | `crates/qbz-audio/src/analysis/spectral_ribbon.rs` |  |
| 204 | `crates/qbz/src/about.rs` |  |
| 204 | `crates/qbz-audio/src/diagnostic.rs` |  |
| 203 | `crates/qbz/src/myqbz_view_prefs.rs` |  |
| 203 | `crates/qbz/src/musician.rs` |  |
| 202 | `crates/qbz-app/src/user_data.rs` |  |
| 200 | `crates/qbz-playlist-import/src/providers/deezer.rs` |  |
| 199 | `crates/qbz-reco/src/weights.rs` |  |
| 198 | `crates/qbz-qobuz/src/retry.rs` |  |
| 197 | `crates/qbz/src/device_cap.rs` |  |
| 196 | `crates/qbzd/src/tui/screens/account.rs` |  |
| 195 | `crates/qbzd/src/api/fav.rs` |  |
| 193 | `crates/qbz-theme/src/color.rs` |  |
| 191 | `crates/qbz-dsd/src/native.rs` |  |
| 191 | `crates/qbz-cmaf/src/crypto.rs` |  |
| 190 | `crates/qbz-secrets/src/lib.rs` |  |
| 189 | `crates/qbzd/src/cli/scrobble.rs` |  |
| 186 | `crates/qbz/src/search_service.rs` |  |
| 185 | `crates/qbz-i18n/src/plural.rs` |  |
| 185 | `crates/qbzd/src/state.rs` |  |
| 185 | `crates/qbz-audio/src/output_sinks.rs` |  |
| 185 | `crates/qbz-app/src/graphics_autoconfig.rs` |  |
| 184 | `crates/qbz/src/share.rs` |  |
| 184 | `crates/qbzd/src/qconnect/publish.rs` |  |
| 183 | `crates/qbz-music-link/src/lib.rs` |  |
| 181 | `crates/qbz-audio/src/jack_backend.rs` |  |
| 176 | `crates/qbz-theme/src/lib.rs` |  |
| 175 | `crates/qbz-cast/src/chromecast/discovery.rs` | partira avec REMOVAL-SPEC.md |
| 174 | `crates/qbz-external-reco/src/lib.rs` |  |
| 172 | `crates/qbzd/src/paths.rs` |  |
| 172 | `crates/qbzd/src/config.rs` |  |
| 169 | `crates/qbz/src/single_instance.rs` |  |
| 169 | `crates/qbz-secrets/src/backend.rs` |  |
| 168 | `crates/qbz-theme/src/colors.rs` |  |
| 168 | `crates/qbz-library/src/thumbnails.rs` |  |
| 168 | `crates/qbz-library/src/tag_writer.rs` |  |
| 167 | `crates/qbz-dsd/src/dsd2pcm.rs` |  |
| 166 | `crates/qbz-offline-cache/src/playback.rs` |  |
| 165 | `crates/qbz-library/src/tag_sidecar.rs` |  |
| 164 | `crates/qconnect-app/src/startup.rs` |  |
| 164 | `crates/qbz/src/location_view.rs` |  |
| 163 | `crates/qconnect-app/src/events.rs` |  |
| 163 | `crates/qbz-radio/src/tests.rs` | partira avec REMOVAL-SPEC.md |
| 161 | `crates/qbz-music-link/src/qobuz_search.rs` |  |
| 160 | `crates/qbzd/src/api/discover.rs` |  |
| 159 | `crates/qbz-qobuz/src/forbidden_breaker.rs` |  |
| 158 | `crates/qbz-playlist-import/src/lib.rs` |  |
| 158 | `crates/qbz-offline-cache/src/state.rs` |  |
| 158 | `crates/qbz-music-link/src/fast_path.rs` |  |
| 156 | `crates/qbzd/src/api/sse.rs` |  |
| 155 | `crates/qbz/src/play_history.rs` |  |
| 155 | `crates/qbz-models/src/playback.rs` |  |
| 154 | `crates/qbzd/src/tui/mod.rs` |  |
| 153 | `crates/qbz-cast/src/dlna/discovery.rs` | partira avec REMOVAL-SPEC.md |
| 149 | `crates/qbz/src/auto_theme.rs` |  |
| 149 | `crates/qbz-models/src/lib.rs` |  |
| 147 | `crates/qconnect-core/src/queue.rs` |  |
| 147 | `crates/qbz-offline-cache/src/types.rs` |  |
| 146 | `crates/qbz-external-reco/src/types.rs` |  |
| 144 | `crates/qconnect-protocol/src/event.rs` |  |
| 144 | `crates/qbz-offline-cache/src/maintenance.rs` |  |
| 144 | `crates/qbz-log/src/redact.rs` |  |
| 142 | `crates/qbz/src/playlist_snapshot.rs` |  |
| 140 | `crates/qbzd/src/cli/play.rs` |  |
| 140 | `crates/qbz-audio/src/visualizer/tapped_source.rs` |  |
| 137 | `crates/qbz-audio/src/dynamic_amplify.rs` |  |
| 136 | `crates/qbz-playlist-import/src/importer.rs` |  |
| 134 | `crates/qbz/src/library_by_artist.rs` |  |
| 134 | `crates/qbz-mixtape/src/schema.rs` |  |
| 133 | `crates/qbz/src/selection.rs` |  |
| 133 | `crates/qbz-models/src/events.rs` |  |
