# Audit lecture seule — downloader.rs / cmaf_store.rs / migration.rs

Scope : `crates/qbz-offline-cache/src/downloader.rs` (760L), `cmaf_store.rs` (238L), `migration.rs` (261L).
Aucune modification, aucune compilation.

## 1. Parallélisme des téléchargements

- **3 téléchargements de morceaux simultanés max** — `Arc::new(Semaphore::new(3))` dans `crates/qbz-offline-cache/src/state.rs:55` et `:74` (`cache_semaphore`), acquis par `spawn_track_cache_download` (`downloader.rs:472-494`) avant de lancer le download d'un morceau.
- À l'intérieur d'un même morceau, le chemin CMAF (chemin par défaut, "CMAF-first") télécharge ses segments avec un **second niveau de parallélisme = 3** : `CMAF_PREFETCH_CONCURRENCY: usize = 3` (`crates/qbz-qobuz/src/cmaf.rs:46`), utilisé par `fetch_all_segments` (`cmaf.rs:423-478`).
- Donc jusqu'à **3 morceaux × 3 segments = 9 requêtes HTTP CDN en vol** en même temps dans le pire cas.

## 2. Risque mémoire pic — chemin CMAF (celui utilisé en pratique)

**Oui, risque réel et confirmé.** Le chemin CMAF-first (`try_cmaf_offline_download`, `downloader.rs:264-457`, appelé en premier par `spawn_track_cache_download` avant tout fallback legacy) **bufferise le morceau entier en RAM avant d'écrire quoi que ce soit sur disque** :

- `fetch_all_segments` (`crates/qbz-qobuz/src/cmaf.rs:423-478`) spawn une tâche tokio par segment (1..=n_segments) dès le départ. Le semaphore à 3 ne limite que les requêtes HTTP *en vol*, pas la rétention mémoire : chaque segment déjà téléchargé reste dans son `handle` en attendant que **tous** les segments du morceau soient reçus (`for handle in handles { … }`, ligne 469-475), avant de trier et retourner `Vec<Vec<u8>>`.
- `download_raw_with_progress` (`cmaf.rs:291-362`) retourne un `CmafRawBundle` qui contient `segments: Vec<Vec<u8>>` — **la totalité du morceau chiffré tient en mémoire d'un coup**.
- Ce n'est qu'ensuite que `crate::cmaf_store::persist_bundle` (`downloader.rs:349`, implémenté en `cmaf_store.rs:76-140`) écrit sur disque — et à ce stade `persist_bundle` reçoit déjà `&bundle` en entier, donc le buffer RAM existe déjà avant l'appel.
- Conséquence : pour un FLAC 24-bit/192 kHz (~150-400 Mo/morceau selon durée), avec 3 morceaux en parallèle (`cache_semaphore`), le pic mémoire peut atteindre l'équivalent de **3 fichiers hi-res entiers simultanément**, avant même de compter :
  - le vault wrapping (`content_key` — négligeable, mais `infos.as_bytes()` copie),
  - la persistance CMAF elle-même (`persist_bundle` réutilise les `&bundle.segments` déjà en RAM plutôt que de les libérer au fur et à mesure de l'écriture — le `BufWriter` écrit dans `segments.bin` mais les `Vec<Vec<u8>>` sources restent vivants jusqu'à la fin de la fonction appelante).
  - `LoadedBundle::decrypt_to_flac` (`cmaf_store.rs:212-226`, chemin lecture/playback, pas download) fait aussi une allocation `Vec::with_capacity(total)` pour le FLAC déchiffré entier — même pattern "tout en RAM", mais hors scope du download.

**Le chemin legacy (`StreamFetcher::fetch_to_file`, `downloader.rs:52-220`) n'a PAS ce problème** : il stream chunk par chunk directement sur disque (`file.write_all(&chunk)` dans la boucle `while let Some(chunk_result) = stream.next().await`, ligne 154-173) sans jamais bufferiser le fichier entier en mémoire. C'est le fallback (`downloader.rs:512-530`), utilisé seulement si CMAF échoue.

**Verdict** : le risque décrit par la question est réel, mais concentré sur le chemin CMAF (celui utilisé en priorité aujourd'hui), pas sur le chemin legacy qui est déjà correctement streamé. Une piste de correction (hors scope de cet audit lecture seule) serait de streamer l'écriture segment par segment dans `fetch_all_segments`/`persist_bundle` au lieu d'attendre la collecte complète — mais cela demande de repenser l'ordre segments-hors-ordre + manifest.

## 3. `migration.rs` — code mort candidat au retrait

- Contenu : migration ponctuelle des anciens fichiers FLAC mis en cache sous nommage numérique plat (`tracks/<id>.flac`, ancien schéma v1 "flat") vers la nouvelle arborescence organisée artist/album (`organize_cached_file`), avec écriture des tags, artwork, et insertion en base bibliothèque. Fonctions publiques : `detect_legacy_cached_files`, `migrate_legacy_cached_files`, structs `MigrationStatus`/`MigrationError`.
- **Aucun appelant actif trouvé** : ces fonctions sont ré-exportées depuis `crates/qbz-offline-cache/src/lib.rs:39` mais ne sont invoquées nulle part ailleurs dans le dépôt — ni par une commande Tauri (`#[tauri::command]` absent du fichier, aucune trace dans les `generate_handler!`/`invoke_handler`), ni par du code frontend (aucune correspondance `migrate_legacy`, `detect_legacy`, `MigrationStatus`, `has_legacy_files` dans les sources front `.ts/.tsx/.slint/.svelte/.vue`).
- Historique git : introduit le 27 mai 2026 (commit `5f882017`, déplacement depuis un ancien module), puis re-branché lors du refactor Tauri (`c8158bf4`) — mais toujours sans point d'entrée UI/commande observable.
- **Conclusion** : tout porte à croire que c'est une migration ponctuelle laissée en place "au cas où" plutôt qu'un chemin actif — candidat clair à suppression (ou au moins à documentation explicite "dead code, safe to remove after confirming no user still on pre-v2 flat layout") si l'équipe confirme que tous les utilisateurs installés sont déjà passés au nouveau layout organisé. Ne pas retirer sans vérifier auprès de l'auteur si un mécanisme de déclenchement existe ailleurs (ex. appel manuel via CLI qbzd, script de support) que la recherche n'aurait pas capté.

## Fichiers consultés
- `/home/vensam/dev/qbz/crates/qbz-offline-cache/src/downloader.rs`
- `/home/vensam/dev/qbz/crates/qbz-offline-cache/src/cmaf_store.rs`
- `/home/vensam/dev/qbz/crates/qbz-offline-cache/src/migration.rs`
- `/home/vensam/dev/qbz/crates/qbz-offline-cache/src/state.rs` (semaphore)
- `/home/vensam/dev/qbz/crates/qbz-qobuz/src/cmaf.rs` (fetch_all_segments, download_raw_with_progress, CMAF_PREFETCH_CONCURRENCY)
- `/home/vensam/dev/qbz/crates/qbz-offline-cache/src/lib.rs` (re-exports)
