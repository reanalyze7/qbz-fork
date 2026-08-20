# Audit — coût des diagnostics/logs (lecture seule)

Périmètre : `crates/qbz/src/diagnostics.rs`, `crates/qbz-app/src/diagnostics.rs`,
`crates/qbz-log/src/redact.rs`, `crates/qbz-ui/ui/shell/LogViewerModal.slint` +
`crates/qbz/src/log_viewer.rs`, `crates/qbz-ui/ui/settings/DiagnosticsPanel.slint`.
Repère amont : `crates/qbz-log/src/{install,tee,ring,line,bundle}.rs`.

## 1. Les logs sont-ils écrits sur disque en continu ?

**Oui, en continu, dès le niveau `info` (le défaut).**

`crates/qbz-log/src/tee.rs` (`TeeLogger::log`) est le point d'écriture unique
de **chaque** `log::info!/warn!/error!/debug!` accepté par le filtre. Pour
chaque ligne, il fait TROIS écritures synchrones :
1. push dans le ring mémoire (`ring::push`, cheap),
2. `writeln!` sur le `BufWriter<File>` du fichier de log (`open_log_file()`
   dans `install.rs`) — **I/O disque bufferisé mais pas rare : à chaque ligne
   loggée en usage normal**,
3. `writeln!` sur stderr — I/O terminal, même fréquence.

Le fichier est ouvert par `qbz::main.rs:8224` avec `qbz_log::install("info")`
(le binaire `qbz` loggue en info par défaut) et par `qbzd/src/daemon.rs:44`
avec le niveau configuré (`cfg.log.level`). Donc en usage normal (niveau
info), toute activité applicative un tant soit peu bavarde (playback events,
réseau, settings) déclenche une écriture disque — ce n'est pas limité aux
erreurs ni au mode debug.

Note : le commentaire dans `install.rs` documente un incident réel (#555) où
zbus/tracing en info produisait des dumps multi-KB par message D-Bus (ex:
MPRIS polling) qui saturaient le sink fichier en ~1s — d'où le `zbus=warn,
tracing=warn` forcé dans le filtre par défaut. Ça confirme que le volume de
logs "normal" est significatif, pas anecdotique.

Le `BufWriter` bufferise en mémoire process mais `flush()` n'est appelé que
sur `TeeLogger::flush()` explicite (pas après chaque `log()` — bon point,
limite le nombre de syscalls `write()` réels), donc le coût est amorti mais
pas nul : verrouillage d'un `Mutex<BufWriter<File>>` à chaque ligne + accès
disque périodique quand le buffer se remplit ou à la fermeture du process.

## 2. Rotation / limite de taille des fichiers de log ?

**Aucune limite de taille. Une seule génération de rotation, au démarrage
seulement.**

`install.rs::open_log_file()` :
- Au lancement, si `qbz.log` existe déjà, il est renommé en `qbz.log.prev`
  (un seul fichier précédent conservé, écrasé à chaque nouveau run).
- Un nouveau `qbz.log` est créé (`File::create`, tronque).
- **Pas de rotation par taille pendant l'exécution** : durant toute la durée
  d'un run, `qbz.log` grossit sans limite. Une session longue (le player
  tourne des jours) avec logging bavard peut produire un fichier volumineux
  sans borne haute ni purge automatique.
- Chemin : `~/.local/share/qbz/logs/qbz.log` (+ `.prev`).

Le ring mémoire, lui, est borné (`RING_CAP = 5000` lignes, FIFO,
`crates/qbz-log/src/ring.rs`) — donc la vue "Log Viewer" reste bornée même si
le fichier disque, non.

## 3. `redact.rs` tourne-t-il sur chaque ligne loggée, ou seulement à l'export ?

**Sur CHAQUE ligne loggée, à tout niveau (info/debug inclus), pas seulement à
l'export.**

`tee.rs::TeeLogger::log()` appelle `redact::redact(&record.args().to_string())`
inconditionnellement pour toute ligne qui passe le filtre `log`
(`self.inner.enabled(record.metadata())`). C'est le "single write choke
point" documenté explicitement dans les doc-comments de `lib.rs`, `tee.rs`,
`line.rs`. Résultat : ring, fichier ET stderr reçoivent tous le même texte
déjà caviardé — pas de caviardage différé à l'export/bundle/upload (ceux-ci
re-caviardent quand même, "defensively", en double travail — voir
`log_viewer.rs::build_share_text`, `bundle.rs::format_diagnostics_bundle`).

Coût par ligne dans `redact::redact()` (`crates/qbz-log/src/redact.rs`) :
- `line.to_string()` — une allocation/clone du message complet.
- Couche 1 (secrets littéraux) : itère sur `Vec<String>` de secrets
  enregistrés (`register_secret`), `.contains()` puis `.replace()` par
  secret. Coût proportionnel au nombre de secrets vivants (généralement
  faible : tokens auth Qobuz) mais s'exécute sur CHAQUE ligne, même sans
  aucun secret dedans.
- **`out.to_ascii_lowercase()` est fait sans condition** sur la ligne
  entière (après la couche 1), donc une deuxième allocation + parcours
  complet de la chaîne à chaque ligne, même pour des lignes qui ne
  contiennent jamais de motif sensible.
- Ensuite un pré-check bon marché (`has_redaction_candidate`, 6 `.contains()`
  sur des mots-clés) court-circuite les 10 regex si aucun candidat — bonne
  optimisation, mais elle intervient APRÈS le `to_ascii_lowercase()`, pas
  avant.

Donc le coût réel par ligne loggée = 1 alloc (to_string) + N `.contains`/
`.replace` (secrets vivants) + 1 alloc+scan complet (lowercase) + 6
`.contains` (pré-check) + regex seulement si candidat trouvé. Ce n'est pas
gratuit à haute fréquence (ex. logs verbeux en boucle audio/réseau) mais
reste borné et sans I/O propre — le coût dominant en usage normal est bien
l'I/O disque/stderr du point 1, pas `redact()` lui-même.

## 4. Diagnostics UI (Slint) — polling / I/O caché ?

**Non, tout est à la demande, sauf le ring-viewer qui ne touche pas le disque.**

- `DiagnosticsPanel.slint` : le refresh (`DiagnosticsState.refresh()`) n'est
  déclenché QUE par clic utilisateur (bouton Refresh) ou au premier
  déploiement du panneau (`if panel-open && !loaded`). Aucun timer, aucun
  polling. Le refresh lit `/proc`, `/sys`, `/etc/os-release`, les stores de
  settings (I/O disque léger mais ponctuel, pas en continu) — voir
  `crates/qbz-app/src/diagnostics.rs` (pur, infaillible, snapshot à la
  demande) et `crates/qbz/src/diagnostics.rs` (contrôleur, `refresh()` async
  spawné sur clic).
- `LogViewerModal.slint` + `crates/qbz/src/log_viewer.rs` : `init` du modal
  appelle `refresh()` une fois à l'ouverture (snapshot du ring mémoire,
  aucune I/O disque). L'auto-tail (`toggle-auto-tail`) relance `rebuild()`
  toutes les 1.5s via un `slint::Timer` — mais ça ne lit QUE le ring mémoire
  (`qbz_log::ring::snapshot()`), jamais le fichier sur disque. Donc l'UI de
  logs n'ajoute aucune I/O disque récurrente au-delà de ce que `TeeLogger`
  écrit déjà en continu côté logging lui-même.
- `open-log-file` (bouton "Open log file") ouvre le fichier avec l'appli
  système par défaut (`open::that`), pas de lecture par qbz.
- `upload` / `copy-bundle` sont strictement à la demande (clic), font une
  requête réseau POST vers `paste.rs` uniquement sur clic "Upload".

## Résumé des risques identifiés

1. **I/O disque continue et non bornée en usage normal** (niveau info par
   défaut) : chaque ligne loggée déclenche potentiellement une écriture
   fichier, sans rotation par taille pendant le run — seulement un swap au
   démarrage suivant. Un run long et bavard peut produire un `qbz.log`
   volumineux.
2. **`redact()` fait un `to_ascii_lowercase()` inconditionnel sur toute la
   ligne**, avant le pré-check de candidature — coût CPU systématique à
   chaque ligne loggée (info/debug compris), même pour des lignes qui n'ont
   jamais besoin de caviardage.
3. Le ring mémoire est bien borné (5000 lignes) et l'UI (panel + log viewer)
   n'ajoute aucune I/O disque en plus — tout y est on-demand/à clic, sauf
   l'auto-tail qui ne touche que la mémoire.
