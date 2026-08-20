# Audit légèreté — pipeline playback (qbz)

Lecture seule, aucun fichier modifié, aucune compilation. Périmètre :
`crates/qbz-player/src/player/mod.rs` (5617L), `playback_engine.rs` (981L),
`streaming_source.rs` (1206L), `crates/qbz/src/playback.rs` (5395L).

Contrainte respectée : aucune piste ne touche au DSP / au chemin bit-perfect
ALSA-Direct. Tout ce qui touche à ce cœur est classé NE PAS TOUCHER même quand
la structure semble redondante.

## 1. Trouvaille sûre — duplication structurelle réelle (streaming_source.rs)

`IncrementalStreamingSource` (L566-777, réseau/téléchargement incrémental)
et `InMemorySource` (L853-1013, bytes déjà en RAM) sont **quasi
identiques structurellement** :

- Mêmes champs : `sample_rate`, `channels`, `sample_queue: VecDeque<f32>`,
  `format: Box<dyn FormatReader>`, `decoder: Box<dyn Decoder>`, `track_id`,
  `finished`.
- `new()` : même séquence probe Symphonia → `default_track()` → extraction
  sample_rate/channels → `get_codecs().make(...)` — copié-collé avec juste le
  message d'erreur qui change.
- `seek_to()` : **identique** (seek Accurate/SeekTo::Time, reset decoder,
  clear queue, reset finished).
- `impl Source` (`current_span_len`/`channels`/`sample_rate`/`total_duration`) :
  **identique** dans les deux.
- `impl Iterator::next()` : **byte-pour-byte identique** (L800-812 vs
  L1005-1013) — même calcul de `min_buffer`, même appel à `decode_more`, même
  `pop_front`.
- `decode_more()` : quasi identique ; la seule vraie différence est que la
  version streaming gère en plus `SymphoniaError::IoError(WouldBlock)` pour
  attendre les données réseau pas encore arrivées (retry 5ms + comptage
  underrun via `network_throttle`). La version in-memory n'a pas ce cas
  puisque tout est déjà en RAM.

**Ce qui est réellement différent entre les deux** : uniquement la source
`Read`/`Seek` sous-jacente — `BufferedMediaSource` (buffer qui grossit,
condvar, blocage réseau) vs `InMemoryMediaSource` (simple `Cursor<Vec<u8>>`).
Le décodage Symphonia, le seek, le calcul de tampon et l'itération sur les
échantillons sont du pur boilerplate dupliqué autour de ce point de variation.

**Piste d'unification (mécanique, sans toucher au DSP)** : un seul type
générique `SymphoniaSource<R: Read + Seek>` (ou trait commun) paramétré par
la source I/O, avec le cas `WouldBlock` géré de façon générique (no-op pour
un `Cursor` qui ne le renvoie jamais). Le `new()`, `seek_to()`, `impl Source`
et `impl Iterator` deviendraient un seul jeu de code au lieu de deux. Aucune
formule de décodage, de resampling ni de traitement du signal ne change —
c'est une fusion de deux wrappers identiques autour de Symphonia, pas une
modification de la chaîne audio.

Risque estimé : faible mais **pas nul** — `decode_more` de la version
streaming a un chemin `WouldBlock` avec effet de bord sur
`network_throttle::state().record_underrun()` qui doit être préservé
exactement. Une IA ou un dev non-audio ne doit pas fusionner ça sans tests de
non-régression sur le streaming réseau (coupures/relances). Recommandé
**avec** review humaine avant merge, mais safe à investiguer.

## 2. `BufferedMediaSource` vs `InMemoryMediaSource` (même fichier)

Même constat à un niveau en dessous : `impl Read`/`impl Seek`/
`impl MediaSource` pour les deux structs. Ici en revanche la logique
`BufferedMediaSource::read/seek` (condvar, attente de données, gestion
`download_error`/`download_complete`) est substantiellement plus complexe que
le simple délégué `Cursor` de `InMemoryMediaSource` — la duplication est
beaucoup plus fine (surtout la forme des impls de traits, pas leur corps).
Moins rentable à unifier que le point 1 ; **classé bas priorité**, pas
d'action recommandée.

## 3. `playback.rs` — les 5 plus grosses fonctions

| Lignes | Fonction | Rôle |
|---|---|---|
| 771 | `start_poll_loop` (L4624) | Boucle de poll 450ms unique : toast d'erreur stream, réflexion d'état QConnect peer, cast, MPRIS, sauvegarde position session, prefetch gapless, push UI. |
| 478 | `refresh_now_playing_meta` (L1895) | Reconstruit tout le méta now-playing (titre/artiste/album/artwork/contexte) à partir de l'état de queue et le pousse vers UI/MPRIS/tray/lyrics. |
| 196 | `play_audible` (L570) | Dispatcher central : offline fast-fail → cast → QConnect peer → **dispatch par `source`** (local/plex/qobuz) → gestion d'erreur (403, indispo terminale, auto-skip). |
| 173 | `play_track_in_context` (L3414) | Point d'entrée "jouer ce morceau dans tel contexte" (album/playlist/artiste/mix) — construit la queue puis appelle le dispatcher. |
| 143 | `play_local_file_audible` (L819) | Chemin fichier local : CUE fast-path, DSD, lecture disque + `play_data`. |

**Verdict : complexité légitime, pas de duplication évidente.**
`play_audible` (L570) est déjà le pattern correct — un seul dispatcher qui
route vers `play_local_file_audible` / `play_plex_audible` / le chemin Qobuz
par défaut selon `qt.source`. Ce n'est pas trois façons parallèles de
streamer qui pourraient fusionner : chaque branche a une mécanique d'I/O
réellement différente (lecture fichier + `play_data` vs résolution Plex +
streaming progressif vs `play_track_resolved` Qobuz), et le dispatch lui-même
n'est pas dupliqué.

`start_poll_loop` et `refresh_now_playing_meta` sont de vrais monolithes
(771L et 478L pour UNE fonction), mais leur taille vient de la quantité de
préoccupations qu'ils orchestrent (QConnect, cast, MPRIS, tray, lyrics,
session persist, dirty-guards anti-repaint), pas de code répété plusieurs
fois. Une extraction en sous-fonctions nommées (mécanique, sans changer la
logique) améliorerait la lisibilité mais **touche à du code qui pilote
position/lecture temps réel** → classé **NE PAS TOUCHER — nécessite un
humain expert audio/UI** pour la réorganisation, même si le risque semble
d'abord "juste du découpage".

## 4. `mod.rs` (qbz-player) — pas de piste sûre

- `pub fn new(...)` (L1354) fait à elle seule **2672 lignes** : c'est le
  constructeur du moteur audio (setup device, callbacks cpal/ALSA/JACK,
  threads). Aucune duplication détectée — c'est un seul flux d'init
  monolithique. **NE PAS TOUCHER**, cœur du pipeline bit-perfect.
- `try_init_stream_with_backend` (214L), `create_output_stream_with_config`
  (121L) : sélection de backend audio (ALSA direct / cpal / JACK). Pourrait
  *sembler* dupliquer de la logique entre backends mais c'est exactement le
  genre de code où une fausse unification casserait le chemin ALSA-Direct.
  **NE PAS TOUCHER — nécessite un humain expert audio.**

## 5. `playback_engine.rs`

`alsa_writer_thread`, `jack_feeder_thread`, `dop_writer_thread` : trois
threads d'écriture bas niveau, un par backend de sortie (ALSA direct, JACK,
DoP/DSD-over-PCM). Structure superficiellement similaire (thread qui pop une
queue et écrit vers le device) mais chaque backend a une API et des
contraintes de timing différentes. **NE PAS TOUCHER — cœur du pipeline
audio**, aucune conclusion de sécurité possible sans expertise audio dédiée.

## Résumé

Une seule piste "sûre" identifiée avec confiance raisonnable : la duplication
quasi totale entre `IncrementalStreamingSource` et `InMemorySource` dans
`streaming_source.rs` (boilerplate Symphonia — init/seek/Source/Iterator
identiques), point 1 ci-dessus. Tout le reste examiné (dispatcher
`play_audible`, poll loop, constructeur du moteur audio, threads writer par
backend) est soit déjà correctement factorisé (le dispatcher), soit une vraie
complexité multi-device/multi-source qui ne doit pas être touchée sans un
expert audio dédié. Pas de deuxième ou troisième piste supplémentaire trouvée
avec un niveau de confiance suffisant — je ne force pas de trouvailles
au-delà de celle-ci.
