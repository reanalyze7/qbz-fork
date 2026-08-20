# Audit — coût CPU du visualiseur spectral (lecture seule)

Périmètre : `crates/qbz-audio/src/visualizer/{mod.rs,processor.rs,ring_buffer.rs,tapped_source.rs}`,
`crates/qbz/src/visualizer.rs`. Aucune modification, aucune compilation.

## Verdict bit-perfect : PAS DE RISQUE

`TappedSource::next()` (tapped_source.rs:48-58) retourne toujours
`Some(self.inner.next()?)` **inchangé**. Le push vers le ring buffer
(`self.ring_buffer.push(sample)`) est un effet de bord sur une copie séparée
(`ring_buffer.rs`), jamais une mutation de la valeur renvoyée au flux de
lecture. `RingBuffer` est un SPSC lock-free documenté explicitement
"CRITICAL: this must not affect bit-perfect playback in any way" — pas de
lock, pas d'allocation dans `push`/`snapshot`, un seul `AtomicUsize`
(`Ordering::Relaxed`), écriture via `UnsafeCell` non bloquante. Le flux qui
part vers `engine.append(source)` (donc vers ALSA-Direct / le DAC) est
exactement le flux d'entrée, échantillon pour échantillon. La dérivation est
bien read-only, comme le suggère le nom : elle "tape" une copie, jamais le
flux lui-même.

## Le traitement FFT lourd est bien gated sur la visibilité réelle

- `VisualizerTap.enabled` démarre à `false` (`mod.rs:57`).
- `crates/qbz/src/visualizer.rs:71-75,296-316` : le tap n'est activé que
  quand `VisualizerState::set-enabled(true)` est appelé côté Slint — au
  commentaire du fichier : "the immersive view calls
  `VisualizerState::set-enabled(true)` on open". Donc en barre de lecture
  normale (sans mode Large/Immersive), `enabled = false`.
- `processor.rs:128-139` (`run_fft_loop`) : tant que `!enabled ||
  paused`, le thread FFT **parke** (`park_timeout(200ms)`) au lieu de
  boucler à `TARGET_FPS` (30 Hz) — donc pas de Hann window, pas de FFT
  (`samples_fft_to_spectrum`), pas de calcul des 5 bandes d'énergie, pas de
  détection de transitoire, pas de downsample waveform tant que rien
  n'affiche le spectre.
- Le drain UI 30 fps côté Slint (`visualizer.rs:124-284`) est lui aussi
  démarré/arrêté avec le tap (`timer.stop()` / `timer.restart()`,
  `visualizer.rs:289,305-313`) — pas de tick UI-thread en pure lecture non
  plus.
- Bonus : même quand le tap est *enabled* mais que la lecture est en pause,
  `paused` (mis à jour en miroir de `NowPlayingState`) fait reparker le
  producteur plutôt que de re-FFT un buffer figé (`mod.rs:41-47`,
  `processor.rs:129-137`).

Donc : **aucune analyse spectrale ne tourne** hors du mode Large/Immersive
ouvert. Le calcul FFT/DSP est bien gated sur la visibilité réelle, pas
seulement sur "un morceau joue".

## Le seul coût permanent : un atomic load par échantillon, toujours

`TappedSource` est inséré **inconditionnellement** en tête de pipeline
(`crates/qbz-player/src/player/mod.rs:1428-1437`, commentaire "Visualizer
tap (outermost)") dès que `thread_viz_tap` est `Some` — et il l'est
systématiquement, car `main.rs:8768` construit toujours le runtime via
`AppRuntime::with_visualizer(...)`. Il n'existe pas de code path qui
construise le runtime sans tap.

Conséquence : à chaque échantillon audio, y compris en barre de lecture
normale sans spectre affiché, `TappedSource::next()` exécute un
`self.enabled.load(Ordering::Relaxed)` (tapped_source.rs:52) avant de
décider de pousser ou non dans le ring buffer. C'est un coût réel mais
minime — un atomic load relaxed, inline (`#[inline]` sur `next()`), sans
branche coûteuse, sans allocation, sans lock. Comparé au FFT (qui, lui, est
bien gated), ce coût est négligeable en pratique, mais ce n'est
techniquement pas "zéro traitement visualiseur" quand rien ne s'affiche : le
wrapper structurel reste présent dans le chemin vers le DAC en permanence.

Piste d'allègement possible **sans toucher au flux audio** : ne construire
`TappedSource` autour de la source que si `tap.enabled` est vrai **au moment
du wrap**, ou reconstruire le pipeline sans le wrapper quand le tap est
définitivement désactivé pour la session — mais cela demande de recréer la
chaîne de sources à l'activation/désactivation de la vue Immersive, ce qui
touche à la construction du flux qui va au DAC. **Signalé mais non
recommandé sans revue supplémentaire : toute modification de l'ordre/de la
composition du pipeline de sources est une zone sensible pour le
bit-perfect, même si le patch lui-même semble anodin.**

## Résumé

| Composant | Tourne en permanence ? | Gated sur affichage réel ? |
|---|---|---|
| `TappedSource` (wrapper de flux) | Oui — présent dans le pipeline dès qu'un morceau joue | Non — toujours inséré ; seul le `push()` interne est gated par `enabled` |
| Push vers `RingBuffer` | Non | Oui — no-op si `enabled == false` |
| Thread FFT (`run_fft_loop`) | Non — parke à 200ms tant que disabled/paused | Oui — activé uniquement à l'ouverture du mode Large/Immersive |
| Drain UI 30fps (Slint timer) | Non — stoppé/relancé avec le tap | Oui |

Aucun risque sur le bit-perfect : la dérivation est strictement en lecture
seule vers une copie séparée, le flux vers le DAC n'est jamais modifié. Le
seul coût "always-on" est un atomic load par échantillon (négligeable),
pas un traitement DSP.
