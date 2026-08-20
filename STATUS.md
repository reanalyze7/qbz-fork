# État du fork — 20/08/2026

Dernier commit sur `main` (poussé sur `origin`=git.nas.local ET `github`=reanalyze7/qbz-fork) :
`e993defa` — fix pencil.svg/pen-line.svg.

## Ce qui est fait

REMOVAL-SPEC.md entièrement exécuté et vérifié fichier par fichier (grep sur
toute l'arbo) : suppression totale de Paroles (Lyrics), Cast, Qobuz Connect,
tous les modes de lecture sauf Large, Plex, LastFM/Discogs/Discord,
Radio, Purchases, Awards, Discography Builder. 8 crates + 7 dépendances
orphelines retirées du workspace `crates/Cargo.toml`.

PLAYLIST-REDESIGN-SPEC.md exécuté : ancien flow "créer/ajouter playlist"
éclaté en 8 fichiers remplacé par le picker unifié façon YouTube Music
(`PlaylistAddModal`/`PlaylistAddList`/`PlaylistAddRow`/`PlaylistCreateRow`).

PlayerBar.slint (921L) et PlayerBarSmall.slint (1000L, depuis supprimé avec
le mode Small) éclatés en composants <130L chacun.

## Bug trouvé et corrigé après coup

Deux agents séparés ont fait la même "fusion de doublon" `pencil.svg` /
`pen-line.svg` en pensant que c'était le même fichier — ce n'était pas le
cas (deux tracés SVG différents). Le merge des deux branches a fait
disparaître les deux fichiers en même temps, cassant la compilation
(`Cannot find image file`). **Corrigé** (commit `e993defa`) : les deux
fichiers sont restaurés depuis leur contenu d'origine (`git show c8ef2a1b:...`).

## Bloquant actuel : CI

Le run GitHub Actions #2 (`32373830761`) a échoué sur l'ancien commit
`4706570c` (avant le fix ci-dessus). **Un simple "Re-run" rejoue le même
SHA** — il faut déclencher un tout nouveau run pour prendre le fix :

→ **github.com/reanalyze7/qbz-fork → Actions → "Build Slint (linux)" →
bouton "Run workflow" (pas "Re-run failed jobs") → branche `main`.**

Compilation impossible en local sur cette machine (15 Go RAM, `qbz-ui` seule
a besoin de >18 Go RAM+swap cumulés — confirmé OOM même avec cgroup
memory-capé). Le CI GitHub public (16 Go/4 vCPU sur repo public) est donc
le seul chemin de vérification de compilation.

## Reste à faire (explicitement en attente, ne rien lancer sans confirmation)

- Relancer le CI (ci-dessus) et vérifier qu'il compile en entier cette fois.
- Une fois compilé : télécharger le binaire (artifact du run), l'installer
  en local, smoke-test manuel de chaque flux touché aujourd'hui (playerbar,
  picker playlist, tous les écrans dont une feature vient d'être retirée —
  vérifier qu'aucun bouton/menu mort ne reste visible).
- Décision produit en attente : mode autoplay "Infinite Radio" — cassé
  silencieusement par la suppression de `qbz-radio` (son moteur de
  ré-alimentation de file n'existe plus). Abandonner le mode, ou trouver un
  autre mécanisme de ré-alimentation ?
- TODO-fork.md items 8 (règle 130L sur le reste de l'appli, ~340 fichiers
  restants — voir `FILES-OVER-130-LINES.md`) et 9 (passe de simplification
  visuelle) : **non commencés**, en attente d'une décision de ta part avant
  de lancer un agent dessus.
- `LIGHTNESS-AUDIT*.md` (17 rapports) : pistes d'allègement identifiées
  (caches sans éviction, `tokio` features trop larges, settings dupliqués
  11×...) — aucune implémentée, en attente de priorisation.
