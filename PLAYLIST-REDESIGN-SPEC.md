# Spec de consolidation — création / ajout à une playlist

But : remplacer les allers-retours entre modals séparés par un flux unique
façon YouTube Music : depuis n'importe quel menu contextuel de morceau, un
picker inline liste les playlists (case à cocher = présence du morceau) avec
un raccourci "Nouvelle playlist" en un clic, sans quitter le menu.

Ce document ne fixe aucun style visuel (couleurs, espacements, rayons) —
uniquement la structure des composants et le flux d'interaction. Il ne
contient aucun code, aucune modification de fichier `.slint`/`.rs`.

## 1. Verdict par fichier existant

### Vrai doublon "créer/ajouter une playlist" — cible de la fusion

| Fichier | Lignes | Rôle actuel | Verdict |
|---|---|---|---|
| `primitives/CreatePlaylistModal.slint` | 285 | Formulaire complet (nom, description, dossier, public, offline-only) ouvert depuis le "+" de la sidebar | **Remplacer** — son formulaire "nom seul" migre dans la sous-vue de création inline du nouveau picker ; description/dossier/public restent éditables après coup via Edit (déjà supporté) |
| `primitives/PlaylistPickerModal.slint` | 627 | "Add to Playlist" — sélection UNIQUE dans un dropdown + création inline confirmée par le bouton du footer | **Remplacer** — c'est le point d'entrée central du nouveau flux, mais son interaction "select puis bouton Add" doit devenir une liste à cases à cocher qui agit immédiatement (sémantique YouTube Music : coché = present dans la playlist) |
| `primitives/EditPlaylistModal.slint` | 192 | Renommer / description / offline-toggle / supprimer, ouvert depuis le header de la page détail playlist | **Garder tel quel** — ce n'est pas un doublon : c'est la gestion des métadonnées d'une playlist déjà créée, un concept produit différent ("paramètres de la playlist" vs "ajouter un morceau à une playlist"). Reste le seul endroit pour dossier/public/desc après la création rapide |
| `primitives/PlaylistDuplicateConfirmModal.slint` | 129 | Sous-modal déclenché après un ajout en masse quand des morceaux sont déjà dans la playlist cible | **Garder tel quel** — règle métier réelle (dédoublonnage), indépendante de l'UI du picker. Se raccroche au nouveau flux sans changement : toujours déclenché par une action d'ajout en masse (ex. "ajouter tout l'album" depuis le picker), toujours un sous-modal empilé au-dessus |

### Concepts produit distincts — hors périmètre de la fusion

| Fichier | Lignes | Rôle actuel | Verdict |
|---|---|---|---|
| `primitives/PlaylistImportModal.slint` | 562 | Import d'une playlist externe (Spotify/Apple/Tidal/Deezer) : détection de provider, preview, matching, barre de progression, log, résumé | **Garder tel quel** — fonctionnalité réelle et non triviale (parsing d'URL, matching de morceaux, tâche asynchrone longue), aucun rapport structurel avec "ajouter un morceau à une playlist". Partage seulement la recette visuelle du champ dossier avec `CreatePlaylistModal` (`QbzSelect` + `folder-options`), pas sa logique |
| `myqbz/AddToMixtapeModal.slint` | 553 | "Add to Mixtape/Collection" — ajoute un item (album/morceau/playlist) à une Mixtape ou une Collection MyQBZ, avec création inline | **Garder tel quel** — Mixtape/Collection est un modèle de données à part entière (crate `qbz-mixtape`, table dédiée, pas de lien avec les playlists Qobuz/locales). Une Collection ne contient que des albums entiers ; une Mixtape alimente un tirage aléatoire (`MyQbzMixModal`). Ce fichier est en revanche une **bonne référence structurelle** pour le nouveau picker de playlists : il combine déjà liste + recherche + case "déjà ajouté" + création inline dans un seul panneau |
| `myqbz/CreateMyQbzModal.slint` | 199 | Création d'une Mixtape ou Collection (nom + choix de type) | **Garder tel quel** — même raison : objet MyQBZ, pas une playlist |
| `myqbz/MyQbzMixModal.slint` | 184 | Tirage aléatoire ("Random queue") d'un sous-ensemble de morceaux d'une Collection/Mixtape, mis en file d'écoute | **Garder tel quel** — fonctionnalité de lecture (shuffle sampler), sans rapport avec la création/l'ajout à une playlist |

### Confirmation du state management sous-jacent

Vérifié côté Rust (`crates/qbz/src/main.rs`, `crates/qbz/src/playlist_picker.rs`,
`crates/qbz/src/playlist_import.rs`, `crates/qbz/src/myqbz_add.rs`,
`crates/qbz/src/myqbz_mix.rs`) et `crates/qbz-mixtape/src/*` :

- `CreatePlaylistState`/`EditPlaylistState`/`PlaylistPickerState`/`DuplicateConfirmState`
  pilotent tous des **playlists** (Qobuz distantes ou locales `library.db`) — un seul
  modèle de données.
- `MyQbzCreateState`/`MyQbzAddState`/`MyQbzMixState` pilotent des **Mixtapes/Collections**
  via `myqbz::create_collection(kind, name)` et le crate `qbz-mixtape` (schéma, repo,
  shuffle, enqueue dédiés) — un modèle de données **différent**, avec ses propres règles
  (Collection = albums entiers uniquement ; Mixtape = source du tirage aléatoire).

Ça confirme la frontière : 4 fichiers playlist à fusionner, 1 fichier playlist à garder
(Edit), 1 sous-modal playlist à garder (Duplicate), 1 fichier playlist hors-scope
(Import), 3 fichiers MyQBZ entièrement hors-scope.

## 2. Nouveaux composants — flux unifié façon YouTube Music

Principe : un seul point d'entrée (`PlaylistAddModal`), ouvrable depuis n'importe
quel menu contextuel de morceau/album, qui affiche directement la liste des
playlists avec case à cocher. Chaque case coche/décoche = ajoute/retire le(s)
morceau(x) immédiatement (pas de bouton de confirmation séparé pour la sélection
existante). La création se fait inline, sans quitter le panneau.

```
primitives/
  PlaylistAddModal.slint       — coquille : backdrop, carte, header, recherche, footer
  PlaylistAddList.slint        — Flickable + états vide/chargement + boucle des lignes
  PlaylistAddRow.slint         — une ligne playlist : case à cocher, nom, compteur, icône locale
  PlaylistCreateRow.slint      — ligne "+ Nouvelle playlist" repliée / dépliée (nom + confirmer)
  PlaylistDuplicateConfirmModal.slint  — INCHANGÉ, toujours empilé par-dessus
  EditPlaylistModal.slint      — INCHANGÉ, toujours ouvert depuis le header playlist
```

### `PlaylistAddModal.slint` (~110 lignes)
Responsabilité : structure de la fenêtre modale (backdrop cliquable pour fermer,
carte centrée, header "Enregistrer dans une playlist" + bouton fermer, champ de
recherche/filtre, montage de `PlaylistCreateRow` en première position puis de
`PlaylistAddList`, footer avec un seul bouton "Terminé" qui ferme — pas de
Cancel/Confirm puisque chaque case agit immédiatement). Remplace
`PlaylistPickerModal.slint` comme point de montage global dans `AppShell`, ouvert
par les mêmes sites d'appel (menu contextuel morceau/album, "+" de la sidebar —
ce dernier ouvre directement `PlaylistCreateRow` déplié).

### `PlaylistAddList.slint` (~90 lignes)
Responsabilité : le `Flickable` scrollable, l'état de chargement (spinner),
l'état vide/"aucun résultat" après filtre, et la boucle `for` instanciant une
`PlaylistAddRow` par playlist. Séparé du shell pour rester sous 130 lignes et
parce que c'est une unité cohésive (liste seule, sans logique de recherche ni
de header).

### `PlaylistAddRow.slint` (~70 lignes)
Responsabilité : une ligne = case à cocher (état "présent dans la playlist"),
nom, compteur de morceaux, icône disque pour les playlists locales (parité avec
l'actuel `DropRow`/`AddPickRow`). Le clic sur toute la ligne (pas seulement la
case) bascule l'état — reprend le pattern zone-cliquable-pleine-largeur déjà en
place dans `AddToMixtapeModal.slint::AddPickRow`.

### `PlaylistCreateRow.slint` (~90 lignes)
Responsabilité : remplace `CreatePlaylistModal.slint` pour le cas courant.
Replié : une ligne "+ Nouvelle playlist" toujours en tête de liste (jamais
filtrée, parité avec le comportement actuel de `PlaylistPickerModal`). Cliquée :
se déplie en un champ nom + un bouton de confirmation (Enter ou clic) qui crée
ET coche la playlist dans la foulée — un seul aller. Les champs avancés
(description, dossier, public, offline-only) ne sont **pas** repris ici :
ils restent réglables ensuite via `EditPlaylistModal`, qui les gère déjà tous.
Le "+" de la sidebar peut ouvrir `PlaylistAddModal` avec ce composant pré-déplié
pour retrouver l'ancien raccourci "créer sans ajouter de morceau".

### Composants inchangés réutilisés tels quels
- `PlaylistDuplicateConfirmModal.slint` — reste le sous-modal déclenché par une
  action d'ajout en masse (ex. ajouter un album entier) qui trouve des morceaux
  déjà présents dans la playlist ciblée. Aucune modification structurelle : il se
  raccroche à `PlaylistAddRow`'s "case cochée → ajoute" exactement comme il se
  raccrochait à l'ancien bouton "Add".
- `EditPlaylistModal.slint` — reste le seul endroit pour renommer / changer la
  description / le dossier / la visibilité / supprimer une playlist existante.
- `PlaylistImportModal.slint` — hors périmètre, inchangé.
- `myqbz/AddToMixtapeModal.slint`, `myqbz/CreateMyQbzModal.slint`,
  `myqbz/MyQbzMixModal.slint` — hors périmètre, inchangés (Mixtapes/Collections
  restent un objet distinct des playlists).

## 3. Composants existants supprimés / remplacés

| Fichier | Sort |
|---|---|
| `primitives/CreatePlaylistModal.slint` | **Supprimé** — remplacé par `PlaylistCreateRow.slint`, monté à l'intérieur de `PlaylistAddModal.slint` |
| `primitives/PlaylistPickerModal.slint` | **Supprimé** — remplacé par `PlaylistAddModal.slint` + `PlaylistAddList.slint` + `PlaylistAddRow.slint` |

Tous les autres fichiers listés en §1 restent en place sans modification
structurelle.

## 4. Notes pour les agents de suite (implémentation)

- Le state Slint actuel (`PlaylistPickerState`/`PlaylistPickItem`) a une notion
  de sélection UNIQUE (`selected-id`) ; le nouveau flux a besoin d'un état
  MULTI (par playlist : "morceau(x) déjà présent(s) ?" / "en cours d'ajout ?"),
  plus proche de `MyQbzAddRow.already-has` dans `AddToMixtapeModal.slint` que de
  l'actuel `PlaylistPickItem`. À concevoir côté Rust avant de toucher aux
  fichiers `.slint` ci-dessus.
- Les 8 fichiers existants dépassent tous la règle des 130 lignes/fichier —
  c'est une dette déjà présente, pas introduite par ce document. Les nouveaux
  fichiers proposés y sont dimensionnés dès le départ ; vérifier leur taille
  réelle une fois écrits et re-découper si un composant déborde.
- Aucun fichier n'a été modifié pour produire ce document ; aucune commande
  `cargo build`/`cargo check` n'a été lancée (contrainte mémoire de la machine
  hôte pour ce crate).
