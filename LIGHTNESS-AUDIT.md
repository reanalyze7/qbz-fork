# Audit de légèreté — synthèse (20/08/2026)

17 agents de recherche (lecture seule, aucun code touché, aucune
compilation) ont audité l'app sous l'angle "plus légère, sans jamais
sacrifier la qualité audio bit-perfect". Ce document consolide et priorise
leurs 17 rapports détaillés (`LIGHTNESS-AUDIT-*.md`, mêmes dossier).

**Rien ici n'est exécuté.** Comme pour `REMOVAL-SPEC.md`, l'exécution attend
la confirmation que `main` compile (build CI en cours).

---

## Ce qui touche déjà `REMOVAL-SPEC.md` (déjà intégré là-bas)

- **Chiffrage de l'impact** : ~53 600 lignes retirées (−20,9 % du Slint),
  8/37 crates workspace (−21,6 %), 7 dépendances directes orphelines.
- 2 trous trouvés et corrigés dans `REMOVAL-SPEC.md` : `qbz-plex`/`qbz-radio`
  manquaient à la liste de nettoyage `Cargo.toml` (§5), et les champs
  `Album.awards`/`Album.goodies`/`DiscoverAlbum.awards` manquaient à la ligne
  Awards (§6) — portés par les structs les plus instanciées du projet.
- Couplage croisé Radio↔Purchases (`RadioResponse` dépend de
  `purchase_serde.rs`) à traiter ensemble à l'exécution.

## Priorité 1 — gains réels, risque faible, indépendants de REMOVAL-SPEC

| # | Trouvaille | Gain | Risque |
|---|---|---|---|
| 1 | `tokio = { features = ["full"] }` dans `crates/Cargo.toml`, hérité par 15 crates — l'usage réel ne couvre qu'un sous-ensemble (`rt, rt-multi-thread, macros, sync, time, fs, net, io-util, signal`). 4 crates redéclarent déjà un tokio plus étroit pour contourner ce défaut trop large — la codebase elle-même prouve que `full` est de trop. | Temps de compilation, taille binaire | Faible — feature Cargo, testable en isolant chaque retrait |
| 2 | Cache d'artwork (`qbz-cache/src/image_cache.rs`) : l'éviction LRU ne tourne **qu'au démarrage** (`spawn_evict`, "runs once"), jamais pendant une session longue — peut dépasser sa limite sur plusieurs heures d'écoute. | Mémoire disque sur session longue | Faible — ajouter un déclencheur périodique, pattern déjà existant |
| 3 | `ArtistCacheStore` (`qbz-app/src/settings/search_cache.rs`) : le slice artistes persisté n'a **aucune éviction**, chaque recherche distincte ajoute une clé jamais retirée. | Mémoire + fichier JSON qui grossit sans fin | Faible — même correctif que #2 |
| 4 | `redact.rs` (logs) fait un `to_ascii_lowercase()` inconditionnel sur CHAQUE ligne loggée avant même de vérifier s'il y a un mot-clé sensible à caviarder — coût CPU systématique. | CPU en continu pendant toute la session | Faible — retarder le lowercase après un pré-check rapide |
| 5 | Logs (`TeeLogger`) : pas de rotation par taille pendant l'exécution, seulement un swap au démarrage suivant — le fichier grossit sans borne sur une session longue. | Disque sur session longue | Faible — rotation par taille, mécanisme standard |
| 6 | ~4200 lignes dans `qbz-app/src/settings/` (11 112 lignes au total) sont deux patterns génériques réinventés 11 fois ("keyed set store" SQLite ×4, "singleton settings struct" SQLite ×7), documentés comme copies l'un de l'autre dans les commentaires du code lui-même. | ~4200 lignes → 2 stores paramétrés | Modéré — vrai refactor, à faire quand la compilation sera possible |
| 7 | `IncrementalStreamingSource`/`InMemorySource` (`qbz-player/streaming_source.rs`) : quasi-copié-collé (même construction, seek, `next()`), seule la source I/O change. | Fusionnable en un seul type générique | Modéré — chemin `WouldBlock`/underrun à préserver exactement, review humaine requise |
| 8 | i18n : 8 langues embarquées en dur (`include_str!`, ~887 Ko), aucun moyen d'en exclure au build. RAM correcte (lazy), mais le binaire paie toujours le texte des 8. | ~750 Ko de binaire si on ne garde que le français pour un fork perso | Faible mais **décision produit à prendre** (perdre les autres langues) |
| 9 | 5 icônes SVG mortes (`arrow-left`, `award`, `captions`, `cat`, `lock-open.svg`) + 1 doublon binaire (`pencil.svg`/`pen-line.svg`, identiques, tous deux référencés). | Quelques Ko, négligeable seul mais gratuit | Nul |
| 10 | `migration.rs` (offline downloader) : migration ponctuelle ancien layout → nouveau, aucun appelant trouvé nulle part dans le code. | Suppression pure de code mort | Faible — à confirmer qu'aucun utilisateur n'a encore l'ancien layout non migré |

## Priorité 2 — gains réels mais nécessitent une décision produit ou plus de prudence

| # | Trouvaille | Pourquoi ce n'est pas Priorité 1 |
|---|---|---|
| 11 | CMAF prefetch (téléchargement offline) : un morceau hi-res entier tient en RAM avant écriture disque (`fetch_all_segments`), jusqu'à 3 en simultané. Le chemin legacy, lui, streame correctement vers disque. | Touche au chemin de téléchargement des fichiers hi-res — pas le chemin de lecture bit-perfect, mais proche ; à faire réviser avant de toucher |
| 12 | `panic = "abort"` réduirait la taille du binaire mais casserait le mécanisme de fallback décodeur (`catch_unwind` dans `player/mod.rs::decode_with_fallback`, rodio→mp4→symphonia). | Bloquant tel quel — nécessiterait de d'abord remplacer ce fallback par autre chose |
| 13 | `lto`/`codegen-units = 1` non utilisés — mais les réglages CI actuels (`codegen-units=256`, swap 18G) sont des contraintes MÉMOIRE du runner, pas des choix de taille : les changer va à contre-courant, à tester sur un run CI dédié avant d'adopter. | Nécessite un run CI dédié pour mesurer, pas à l'aveugle |
| 14 | `TappedSource` (visualiseur) est monté inconditionnellement dans le pipeline audio dès qu'un morceau joue, même hors mode spectre — coût négligeable (un atomic load par échantillon) mais présent. | Touche à la composition du pipeline qui va au DAC — zone sensible, non recommandé sans revue même si le risque semble minime |
| 15 | `image` crate (`qbz/Cargo.toml`) n'a pas la feature `webp`, mais plusieurs dialogues UI proposent `.webp` à l'utilisateur — **bug existant**, pas une piste d'allègement, trouvé en marge par l'agent features. | Hors sujet légèreté, mais à corriger un jour (décision produit : ajouter la feature ou retirer `.webp` de l'UI) |
| 16 | D'autres crates (`playlist-import`, `log_viewer`, `qconnect` — ce dernier de toute façon supprimé) recréent un `reqwest::Client` par appel au lieu de le réutiliser — gaspillage TLS handshake. Le client Qobuz principal, lui, est déjà bien fait (pool, cache, retry disciplined). | Périmètre élargi par rapport à la demande initiale, mais signalé pour référence |

## Priorité 3 — résultats négatifs confirmés (rien à faire, vérifié et sain)

Aussi précieux que les trouvailles positives — ces zones ont été activement
vérifiées et jugées déjà correctement conçues, pas juste "non auditées" :

- **Moteur de reco** (`qbz-reco`) : tourne uniquement à la demande, jamais en tâche de fond, structures de données bornées, TTL de cache explicites (7j/48h/9j/30j).
- **Génération de thème auto** : aucun lien avec le morceau en cours, déclenchée uniquement par action utilisateur explicite — zéro coût pendant l'écoute.
- **Visualiseur spectral / FFT** : correctement gated sur la visibilité réelle du composant qui l'affiche, aucun DSP qui tourne hors mode Large/Immersive. Le `TappedSource` (voir Priorité 2 #14) ne touche jamais le flux qui va au DAC.
- **Base de données bibliothèque locale** (6741L) : scan en arrière-plan, index SQLite sur toutes les colonnes de tri/filtre, pas de fonction monstrueuse cachée — la taille vient du nombre de tables (10+), pas de complexité accidentelle.
- **Client réseau Qobuz** : pool de connexion réutilisé, caches internes contre les re-validations réseau, retry discipliné avec backoff + circuit breaker.
- **Dispatcher de lecture** (`playback.rs`) : un seul point d'entrée propre, pas de duplication à corriger.
- **Système d'export/import de settings** (bundle.rs) : fonctionnalité réellement utilisée (bouton UI + CLI `qbzd`), une seule version de schéma — la taille vient de la largeur fonctionnelle légitime (7 domaines), pas de bloat.
- **Frontière `local_library.rs` / `qbz-library`** : saine — le SQL et les types vivent uniquement dans `qbz-library`, sauf l'intégration Plex qui gonfle le fichier UI (mais Plex part déjà via `REMOVAL-SPEC.md`).

---

## Prochaine étape

Rien n'est exécuté. Une fois la compilation confirmée possible (build CI en
cours), trancher : (a) exécuter `REMOVAL-SPEC.md` en premier (le plus gros
morceau), (b) attaquer les gains Priorité 1 de ce document en parallèle ou
après, (c) décider au cas par cas des points Priorité 2 qui demandent un
choix produit (langues i18n, `panic=abort`, LTO).
