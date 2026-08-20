# Audit — `crates/qbz-ui/ui/assets/`

Lecture seule, 20/08/2026. Aucun fichier modifié, aucune compilation lancée.

## 1. Vue d'ensemble

Total dossier `assets/` : **2.3M**

| Sous-dossier | Fichiers | Taille |
|---|---|---|
| `assets/` (racine : logos, badges qualité) | 7 (5 svg + 2 png) | ~292K (dont `mexico-flag.svg` = 156K à lui seul) |
| `assets/icons/` | 130 svg | 608K |
| `assets/fonts/` | 4 ttf (Inter 18pt Regular/Medium/SemiBold/Bold) | 1.4M |

Les fonts (1.4M) dominent le poids total, largement devant les icônes (608K).
Toutes les 4 graisses Inter sont référencées (`app.slint`, `main.rs`,
`lyrics_measure.rs`) — pas de graisse orpheline.

## 2. Fichiers jamais référencés dans le code

Vérifié par `grep -r "nom-fichier.svg" crates/` (hors `crates/target/`
qui contient du code Slint généré et pollue le signal) :

| Fichier | Taille | Statut |
|---|---|---|
| `icons/arrow-left.svg` | 238B | Mort — aucune référence trouvée |
| `icons/award.svg` | 337B | Mort — **et** en plus lié à la feature Awards déjà actée supprimée (§6 REMOVAL-SPEC). `AwardView.slint`/`AlbumPageView.slint` utilisent `laurels.svg` pour l'icône, pas `award.svg`. Double raison de partir. |
| `icons/captions.svg` | 331B | Mort — aucune référence trouvée (probablement une icône envisagée pour les sous-titres/paroles synchronisées puis abandonnée) |
| `icons/cat.svg` | 606B | Mort — aucune référence trouvée |
| `icons/lock-open.svg` | 284B | Mort — `lock.svg` est utilisé (3 fichiers) mais son pendant "déverrouillé" ne l'est pas |

5 fichiers morts sur 137, ~1.8K au total — poids négligeable mais purge facile.

## 3. Doublons visuels probables

| Paire | Constat |
|---|---|
| `icons/pencil.svg` vs `icons/pen-line.svg` | **Doublon confirmé** — `diff` : fichiers strictement identiques (353B chacun). Les deux sont référencés dans le code (4 et 3 fichiers respectivement) → deux noms pour la même icône, un seul devrait suffire. Candidat à fusionner (garder un nom, réécrire les imports de l'autre) une fois hors du chantier lecture-seule actuel. |
| `icons/disc.svg` / `disc-3.svg` / `disc-album.svg` | Trois icônes distinctes (255B/339B/298B) mais usage qui se chevauche sémantiquement (disque/disque tournant/pochette). Toutes les trois référencées activement (9/12/1 fichiers) — pas des doublons pixel-identiques, mais à surveiller si une occasion de consolidation se présente. Pas d'action recommandée ici. |
| `icons/pin.svg` / `pin-filled.svg`, `heart.svg` / `heart-filled.svg`, `folder*.svg` (6 variantes) | Variantes intentionnelles (état filled/outline, ou fonction différente) — toutes activement référencées, pas des doublons. |

Pas d'autre doublon binaire identique trouvé dans `icons/`.

## 4. Fichiers liés à des suppressions déjà actées (REMOVAL-SPEC.md)

Ne pas dupliquer le travail de REMOVAL-SPEC.md — ces fichiers sont déjà
listés là-bas, simple confirmation qu'ils sont bien utilisés aujourd'hui
et partiront avec leur feature :

| Asset | Feature | Statut actuel |
|---|---|---|
| `icons/cast.svg` | Cast (§2) | Utilisé dans 4 fichiers (`CastPicker.slint`, `PlayerBarActionButtons.slint`, `PlayerBarSmallActionButtons.slint`) — pas mort aujourd'hui, partira avec §2 |
| `icons/radio.svg` | Radio (§6) | Utilisé dans 10 fichiers (immersive, artiste, album, menus contextuels) — pas mort, partira avec §6. Note : le dossier `immersive/` part de toute façon entier au §4, donc une partie de ces sites d'appel disparaît déjà par ce biais |
| `icons/plex-logo.svg` | Plex (§6) | Utilisé dans 6 fichiers (cards, glyphes de source) — pas mort, partira avec §6 |
| `laurels.svg` (racine assets, pas dans `icons/`) | Awards (§6) | Utilisé dans `AwardView.slint` et `AlbumPageView.slint` (bloc "AWARDS" du panneau latéral d'album) — partira avec §6. Non listé explicitement dans REMOVAL-SPEC.md § Awards (qui ne cite que `qbz-ui/ui/award/` et `award.rs`) — **à ajouter à la liste d'exécution** quand §6 sera exécuté, sinon fichier orphelin oublié |
| `icons/award.svg` | Awards (§6) | Voir §2 ci-dessus : déjà mort aujourd'hui, indépendamment de la suppression Awards |

Pas d'asset trouvé pour Lyrics (§1) ni Qobuz Connect (§3) — ces features
n'ont pas d'icône SVG dédiée dans `assets/`, seulement des composants
Slint et un bouton texte/icône générique.

## 5. Autres observations (hors périmètre strict mais notables)

- `mexico-flag.svg` fait **153.8K**, soit plus d'un quart du poids
  d'`assets/` racine et plus que toute la fonte Inter Bold — disproportionné
  pour un drapeau utilisé une fois (`AboutModal.slint`, ligne 510, section
  crédits). Probablement un SVG non optimisé (tracé détaillé au lieu de
  formes simples). Pas une suppression, mais un candidat évident à
  ré-exporter/compresser si le poids du binaire devient un sujet.
- `qobuz-logo.svg` (1.2K) vs `qobuz-logo-filled.svg` (18.7K) vs
  `qbz-symbolic.svg` (2.0K) — trois logos distincts, tous utilisés
  (2/7/3 fichiers), pas des doublons.

## 6. Résumé chiffré

- 137 assets audités (130 icônes + 5 racine svg + 2 png), fonts à part (4 ttf)
- 5 fichiers morts identifiés, ~1.8K (`arrow-left`, `award`, `captions`,
  `cat`, `lock-open` — tous dans `icons/`)
- 1 doublon binaire confirmé (`pencil.svg` == `pen-line.svg`)
- 4 assets actuellement utilisés mais voués à disparaître avec
  REMOVAL-SPEC.md (`cast.svg`, `radio.svg`, `plex-logo.svg`, `laurels.svg`)
  — `laurels.svg` n'était pas dans la liste d'exécution, à ajouter
- Fonts (1.4M) toutes utilisées, aucune orpheline
