# TODO — fork local (dimitri/qbz)

Chantier ouvert le 20/08/2026. Objectif global : appli légère, épurée,
rapide, avec deux ajouts fonctionnels (recherche filtrée hi-res, création de
playlist façon YouTube Music). Découpé en ~10 tâches pensées pour être
attribuées à des agents largement indépendants.

## Fait

- [x] `PlayerBar.slint` (921L) éclaté en 15 fichiers ≤130L sous `shell/playerbar/`
- [x] `QBZ_RENDERER=gl` posé (bug de fluidité GPU Intel, pas un problème Slint)

## À faire

1. **Spec — refonte création de playlist façon YouTube Music.** Document de
   conception avant tout code : quel geste unique remplace les 8 fichiers
   actuels (`CreatePlaylistModal`, `EditPlaylistModal`, `PlaylistPickerModal`,
   `PlaylistImportModal`, `PlaylistDuplicateConfirmModal`, `AddToMixtapeModal`,
   `CreateMyQbzModal`, `MyQbzMixModal`) ? Trancher lesquels sont de VRAIS
   concepts distincts (Mixtape/MyQBZ semblent être autre chose qu'une
   playlist) vs pure duplication. **Bloquant pour 2 et 3.**

2. **Implémenter le picker "Ajouter à une playlist"** — popover inline
   depuis n'importe quel menu contextuel de morceau (mirroir du geste YTM :
   liste des playlists + case à cocher + "Nouvelle playlist" en un clic, pas
   un aller-retour vers un modal séparé).

3. **Fusionner Create/Edit en un seul modal** — un formulaire, deux modes
   (création vs édition d'une playlist existante), au lieu de deux fichiers
   séparés qui divergent avec le temps.

4. **Trancher le sort de `AddToMixtapeModal`/`CreateMyQbzModal`/`MyQbzMixModal`**
   selon la décision de la tâche 1 — soit les fusionner dans le flux unifié,
   soit documenter clairement pourquoi ils restent séparés (concept produit
   différent, pas une divergence accidentelle).

5. **Feature — filtre recherche "Hi-Res uniquement".** Toggle dans
   `search/SearchResultsView.slint` + `search/Cortinilla.slint`, câblé sur
   le flag qualité déjà utilisé par `QualityBadge`/`QualityBadgeFull`
   (`primitives/`) côté résultats Qobuz. Vérifier d'abord si l'API Qobuz
   expose ce filtre côté serveur (requête ciblée) ou si c'est un filtrage
   côté client sur les résultats reçus.

6. **Perf — audit du démarrage (launcher).** Profiler le cold-start : quelles
   sous-systèmes s'initialisent avant le premier frame (probe du backend
   audio, scan de bibliothèque locale, assets Immersive/WGSL) qui pourraient
   être différés après l'affichage initial.

7. **Perf/légèreté — alléger le mode Immersive et le mode Kiosk.** Le mode
   Immersive (overlays spectraux WGSL, coverflow, wavebed) et le mode Kiosk
   (vues quasi dupliquées) chargent-ils leurs assets seulement à l'entrée
   dans le mode, ou au démarrage de l'app ? Différer si ce n'est pas déjà
   le cas.

8. **Hygiène de code — appliquer la règle des 130L au reste de l'appli.**
   `PlayerBarSmall.slint` (1000L, jumeau du PlayerBar qu'on vient de découper)
   est le prochain candidat évident. Auditer aussi le reste des 217 fichiers
   `.slint` pour d'autres monolithes.

9. **Design — passe de simplification visuelle.** Épuration des écrans
   restants (densité, chrome redondant) — **a besoin d'une décision de
   direction artistique de ta part avant de lancer un agent dessus**, sinon
   il devine et il faudra recommencer.

10. **Build & vérification finale.** Une fois 1-9 posés : build `--release`
    complet, smoke-test manuel de chaque flux touché, packaging `.deb`,
    installation locale.

## ⚠️ Avant de lancer 10 agents en parallèle

Aujourd'hui, compiler **une seule** crate (`qbz-ui`) a fait planter tes
VSCode une première fois, puis a nécessité un plafond mémoire cgroup pour
échouer proprement au lieu de recrasher — `qbz-ui` seule a besoin de plus de
14 Go de RAM+swap pour compiler. Dix agents qui modifient du code Slint et
relancent chacun `cargo build` en parallèle **sature la machine à coup sûr**,
bien avant même de voir si le code compile.

Je ne lance donc pas 10 agents concurrents sans qu'on ait réglé ce point.
