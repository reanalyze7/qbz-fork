# Plan de découpage — crates/qbz/src/main.rs (22 631 lignes)

Document de préparation uniquement, **aucun code touché**. Décision du
20/08/2026 : on prépare le plan maintenant, l'exécution attend une machine
capable de compiler ce crate (voir journal — cette machine OOM même sur
`qbz-ui` seule avec 18 Go combinés RAM+swap).

## Deux zones de risque très différentes

### Zone A — lignes 166 à 8083 (~7900L) : des fonctions autonomes, risque modéré

131 fonctions `fn`/`pub fn`/`async fn` top-level, groupables par domaine
d'après leur nom :

| Domaine | Fonctions (échantillon) | Cible |
|---|---|---|
| Navigation | `navigate_album`, `navigate_artist`, `navigate_label`, `navigate_search`, `navigate_location`, `navigate_award*`, `navigate_playlist`, `navigate_mix`, `navigate_library_all`, `navigate_purchases`, `navigate_musician`... (~20 fonctions) | `navigation.rs` |
| Favoris / pin / follow | `set_row_favorite`, `set_album_row_favorite`, `set_album_row_pinned`, `set_playlist_row_pinned`, `set_artist_row_pinned`, `toggle_track_favorite`, `playlist_toggle_favorite_by_id`, `playlist_set_follow_by_id`... | `favorites.rs` |
| Mixtape / MyQBZ | `open_add_to_mixtape`, `mixtape_items_from_*`, `wire_myqbz`, `wire_myqbz_detail` (368L + 693L à elles seules) | `mixtape_wiring.rs` |
| Playlist manager | `wire_playlist_manager` (560L à elle seule), `load_sidebar_playlists`, `reconcile_sidebar_after_rename`, `playlist_remove_rows` | `playlist_wiring.rs` |
| Achats (Purchases) | `navigate_purchases`, `load_purchases_tab`, `execute_track_download`, `load_purchase_detail`... | `purchases_wiring.rs` |
| Rendu / GPU / démarrage | `probe_gpu_topology`, `gpu_adapters`, `select_slint_backend`, `detect_hardware_gpu`, `arm_renderer_sentinel`, `arm_startup_probe`... (~25 fonctions, déjà denses) | `renderer_boot.rs` |
| Drag & drop | `local_drag_track`, `row_drag_track`, `gather_drag_tracks` | `drag_helpers.rs` |

**Pourquoi c'est un risque modéré et pas nul** : ce sont déjà des fonctions
séparées (pas de découpage à inventer), mais elles partagent probablement des
imports communs et certaines s'appellent entre elles à travers les domaines
proposés (ex. `navigate_album` peut appeler une fonction de `favorites.rs`).
Chaque extraction doit vérifier ses dépendances croisées avant de bouger quoi
que ce soit — d'où le besoin de compiler pour confirmer qu'aucune référence
n'est cassée.

### Zone B — lignes 8084 à 22631 (~14 500 lignes) : UNE SEULE fonction `main()`, risque élevé

`fn main()` ne s'arrête jamais avant la fin du fichier. D'après l'audit
démarrage déjà fait (`STARTUP-AUDIT.md`), c'est majoritairement de
l'enregistrement de callbacks (`window.on_xxx(...)`) fermant sur des
variables locales partagées (`Rc<RefCell<...>>`, le handle `window` lui-même,
l'état applicatif). **Ça ne se découpe PAS comme la zone A** : on ne peut pas
juste couper-coller des blocs dans des fichiers séparés, parce que chaque
closure capture du contexte local à `main()`.

Stratégie recommandée (à exécuter, pas à deviner) une fois la compilation
possible :
1. Regrouper les `window.on_xxx` par domaine (mêmes domaines que la zone A).
2. Pour chaque domaine, créer une fonction `fn wire_<domaine>(window: &AppWindow, ctx: &SharedCtx) { ... }` qui prend le contexte partagé en paramètre explicite au lieu de le capturer implicitement.
3. `main()` devient une suite d'appels `wire_navigation(&window, &ctx); wire_favorites(&window, &ctx); ...` — cette dernière étape SEULE fait déjà chuter `main()` à quelques centaines de lignes.
4. Compiler après CHAQUE domaine extrait, pas à la fin — un seul domaine mal extrait doit être visible immédiatement, pas noyé dans 14 500 lignes de diff.

## Ce qui n'est PAS dans ce plan

- Aucune estimation de durée — dépend entièrement de la machine de build
  disponible et du nombre d'allers-retours de compilation nécessaires.
- Aucune règle de 130L visée pour la zone B tant que le passage en `wire_*`
  n'est pas fait — c'est un prérequis structurel, pas un simple découpage de
  fichier.
