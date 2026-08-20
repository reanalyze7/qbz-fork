# Audit — settings/bundle.rs (export/import de configuration)

Date : 2026-08-20 · Lecture seule, aucun fichier modifié, aucune compilation.

## Fichiers audités

| Fichier | Lignes |
|---|---|
| `crates/qbz-app/src/settings/bundle.rs` | 1569 |
| `crates/qbz-app/src/settings/bundle/tests.rs` | 576 |
| `crates/qbz-app/src/session_store.rs` | 661 |
| `crates/qbz/src/session_store.rs` | **n'existe pas** — chemin inexact dans la demande |

Total bundle.rs + tests.rs : **2145 lignes**.

## 1. Est-ce relié à un bouton UI réel ?

**Oui, chaîne complète et fonctionnelle, pas du code mort.**

- `crates/qbz-ui/ui/settings/DeveloperSettings.slint` (section « SETTINGS
  PORTABILITY ») expose un `SettingRow` avec bouton **« Export… »** qui met
  `SettingsExportState.open = true`.
- `crates/qbz-ui/ui/settings/SettingsExportModal.slint` est la modale réelle
  (checkbox `--include-auth` défaut OFF, boutons Cancel/Export). Le bouton
  Export appelle `SettingsExportActions.confirm()`.
- Côté Rust, `crates/qbz/src/main.rs:10889` connecte
  `on_confirm` à `export_settings()` défini dans `crates/qbz/src/settings.rs`
  (~ligne 1409), qui appelle directement `bundle::export(...)`, ouvre un
  dialogue natif de sauvegarde (`rfd::AsyncFileDialog`), écrit le fichier
  `.qbzb` en 0600 et toaste la commande d'import.
- Le module `bundle` est **partagé** avec `qbzd` (le CLI daemon) :
  `crates/qbzd/src/cli/settings.rs` (1397 lignes, `qbzd settings show|set` +
  import/export) importe `Bundle`, `ExportOptions`, `ImportPlan`,
  `ProfilePaths`, etc. depuis `qbz_app::settings::bundle`.

Ce n'est donc pas une fonctionnalité isolée : c'est le seul mécanisme de
portabilité de config entre le desktop et le daemon `qbzd`, exposé à la fois
en UI (bouton réel, atteignable en 2 clics depuis Settings > Developer) et en
CLI.

## 2. Combien de versions/formats de bundle en rétrocompatibilité ?

**Une seule** — `SCHEMA_VERSION = 1` (ligne 32), documentée comme « the floor ».

- Le parseur (`build_plan`, step 2, ligne ~414-422) rejette :
  - tout `schema_version < 1` → `BundleError::VersionMalformed`
  - tout `schema_version > SCHEMA_VERSION` → `BundleError::VersionTooNew`
- Aucune branche `match schema_version { 0 => ..., 1 => ... }` : il n'y a
  qu'un seul format supporté, pas de code de migration/rétrocompat legacy.
- Historique git : bundle.rs n'a que **2 commits** depuis sa création
  (`0b7cd6c5` — création initiale « settings bundle engine with
  importer-side classification », `80213939` — fix masquage de secrets dans
  le résumé d'import). Pas d'évolution de format à ce jour.

**Conclusion : la taille du fichier ne vient PAS de la rétrocompatibilité
multi-version.** Elle vient de la largeur fonctionnelle : le module gère 7
domaines de réglages distincts, chacun avec sa propre logique de
classification import (plan) + application (apply) + lecture (export) :

`playback`, `audio` (+ choix de device audio en TTY, `DeviceChoice`),
`prefs` (daemon_prefs), `qconnect`, `integrations` (scrobblers), 
`library_folders`, `auth` (avec gate `--include-auth` et masquage secret).

Le commentaire d'en-tête du fichier pose explicitement l'invariant central
(« CLASSIFICATION LIVES IN THE IMPORTER, NEVER IN THE BUNDLE ») : chaque
domaine a un `plan_<domaine>()` symétrique côté import, ce qui multiplie le
nombre de fonctions plutôt que la complexité par fonction. Les 576 lignes de
`tests.rs` suivent la même logique — 22 tests, un par règle §3/§5 de la spec
citée en commentaire (04-settings-portability.md), et sont désignés comme
« the normative test suite ».

## 3. `session_store.rs` — hors périmètre de bundle.rs

`crates/qbz-app/src/session_store.rs` (661 lignes) est un module **distinct**
et **sans lien** avec `bundle.rs` : c'est la persistance de la file de
lecture (queue) et de l'état de restauration de vue, en SQLite. Il est
consommé activement par `crates/qbz-app/src/shell.rs`,
`crates/qbz-app/src/playback_driver.rs` et `crates/qbz/src/session_persist.rs`
— fonctionnalité de reprise de lecture au redémarrage de l'app, pas de
l'export/import de config. `crates/qbz/src/session_store.rs` mentionné dans
la demande **n'existe pas** dans le dépôt (seul `qbz-app/src/session_store.rs`
existe) — probable confusion de chemin.

## Verdict

Le système export/import de settings **n'est pas une fonctionnalité annexe
morte** : bouton UI réel dans Settings > Developer, chaîne Rust complète et
utilisée en CLI (`qbzd settings`) pour l'interopérabilité desktop↔daemon.
Les 1569+576 lignes ne s'expliquent pas par de la dette de rétrocompat
(un seul schema_version, aucune branche de migration) mais par la largeur
réelle du domaine couvert (7 catégories de réglages × export/plan/apply/test
symétriques). Un axe de réduction plausible, si on veut alléger : fusionner
les fonctions `plan_<domaine>` très similaires, mais ce serait un refactor de
forme, pas une suppression de fonctionnalité inutilisée.
