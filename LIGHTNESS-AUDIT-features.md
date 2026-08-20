# Audit des features Cargo — allègement du binaire

Lecture seule, aucun `Cargo.toml` modifié, aucun build lancé. `symphonia` exclu (déjà audité).

Méthode : pour chaque dépendance avec `features = [...]` (ou sans `default-features = false`), grep des usages réels (`crate::API`) dans le `src/` du/des crate(s) consommateur(s), comparé à la feature list déclarée.

**Rappel qualité audio** : aucun des candidats ci-dessous ne touche à `symphonia`, ALSA, ou au pipeline `qbz-audio`/bit-perfect. `zbus` sur `qbz-audio` (device reservation D-Bus) est examiné uniquement côté surface D-Bus, pas côté chemin audio — voir note dédiée.

---

## Candidat n°1 — `tokio = { features = ["full"] }` au niveau workspace (SÛR, fort impact)

`crates/Cargo.toml:52` :
```toml
tokio = { version = "1", features = ["full"] }
```
Hérité par `tokio = { workspace = true }` dans **15 crates** : `qbz-app`, `qbz-audio`, `qbz` (bin principal), `qbz-core`, `qbz-credentials`, `qbzd`, `qbz-external-reco`, `qbz-lyrics`, `qbz-media-controls`, `qbz-music-link`, `qbz-offline-cache`, `qbz-player`, `qbz-playlist-import`, `qbz-qobuz`, `qbz-reco`, `qconnect-app`, `qconnect-transport-ws`.

Union réelle des usages `tokio::*` sur tout le workspace (grep exhaustif de `src/`) :
`fs::{copy,read,write}`, `io::{AsyncRead,AsyncWrite}`, `join!`, `main`, `net::TcpListener`, `runtime::{Builder,Handle,Runtime}`, `select!`, `signal::ctrl_c` / `signal::unix::*`, `spawn`, `sync::*` (Mutex/RwLock/Notify/Semaphore/broadcast/mpsc/oneshot/watch), `task::{JoinHandle,JoinSet,spawn_blocking,yield_now}`, `test`, `time::*`.

Ça correspond à : `rt`, `rt-multi-thread`, `macros`, `sync`, `time`, `fs`, `net`, `io-util`, `signal`. Aucune trace de `tokio::process`, `io-std`, ni des autres bribes que `full` embarque en plus.

**Preuve interne que le workspace lui-même sait faire plus étroit** : plusieurs crates redéclarent *volontairement* un `tokio` non-workspace plus étroit pour contourner `full` :
- `qbz-cast/Cargo.toml:41` → `features = ["rt", "sync"]`
- `qbz-integrations/Cargo.toml:10` → `features = ["sync", "time"]`
- `qbz-lyrics/Cargo.toml:35` (dev-dependency) → `features = ["macros", "rt", "rt-multi-thread"]`
- `qbz-mixtape/Cargo.toml:35` (dev-dependency) → idem

C'est le signal le plus net de l'audit : le motif exact demandé en exemple (`features = ["full"]` alors qu'un sous-ensemble suffit), documenté par le code lui-même qui contourne le défaut workspace à 4 endroits différents.

**Feature minimale proposée** (à valider par un humain avant tout changement) :
`["rt", "rt-multi-thread", "macros", "sync", "time", "fs", "net", "io-util", "signal"]`

**Confiance : sûr** que `full` dépasse l'usage constaté. **À vérifier par un humain** : l'impact sur le temps de compilation/taille binaire réel (mesure avant/après), et si un crate consommé transitivement (hors de ce repo, ex. dépendance de `rupnp`/`mdns-sd`/`zbus`) exige une sous-feature tokio non détectée par ce grep source-only.

---

## Candidat n°2 — `image` : feature mismatch qui casse du comportement (bug existant, pas juste "trop large")

`crates/qbz/Cargo.toml:58` :
```toml
image = { version = "0.25", default-features = false, features = ["jpeg", "png"] }
```
Or `crates/qbz/src/myqbz_cover.rs:22-23` documente **explicitement** :
> "NOTE on webp: the workspace `image` crate is built with only the `jpeg` + `png` features, so a `.webp` source decodes to an error at runtime"

Pourtant le même fichier et plusieurs autres dans `crates/qbz/src` (`main.rs`, `settings.rs` via file dialogs, `local_library.rs`) proposent/acceptent `.webp` dans les filtres de sélection de fichier utilisateur (`ALLOWED_EXTENSIONS`, `add_filter(...,"webp")`).

Ce n'est pas un candidat d'allègement (sens inverse de la tâche) mais un **mismatch feature ↔ comportement UI** repéré au passage : soit ajouter `"webp"` à la feature list de `qbz/Cargo.toml`, soit retirer `.webp` des filtres de sélection — décision produit, pas une question d'allègement. **À valider par un humain.**

Par comparaison, `qbz-theme/Cargo.toml:18` (`jpeg, png, webp, bmp, tiff`) correspond exactement aux extensions testées dans `qbz-theme/src/auto/system.rs`. `qbz-library/Cargo.toml:31` (`png, jpeg, webp`) et `qbz-media-controls` (`png, jpeg, webp`) semblent alignés avec leur usage (pas de preuve de sur-largeur ni de sous-largeur nette).

---

## Candidat n°3 — `zbus` sur `qbz-audio` : feature `"tokio"` alors que l'usage est 100% `zbus::blocking` (À VÉRIFIER, ne pas changer sans lire le commentaire)

`crates/qbz-audio/Cargo.toml:56` :
```toml
zbus = { version = "4", default-features = false, features = ["tokio"] }
```
Tout l'usage réel dans `crates/qbz-audio/src/device_reservation/linux.rs` passe par `zbus::blocking::{Connection, Proxy, fdo::DBusProxy}` — aucun appel async. Pris isolément, ce crate n'a besoin que de l'API blocking.

**MAIS** le commentaire au-dessus de la ligne dit explicitement que le choix `tokio` sert à "avoid duplicate zbus compiles" — c'est-à-dire une unification volontaire avec un autre usage `zbus` v4 tokio ailleurs dans le graphe de dépendances (potentiellement `mpris-server` dans `qbz-media-controls`, ou `ksni`/`accesskit`). Changer cette feature sans vérifier tout le graphe de résolution `zbus` v4 risque de casser exactement le genre de conflit executor (`async-io` vs `tokio`) que d'autres commentaires du repo (`qbz-credentials/Cargo.toml`, `qbz-media-controls/Cargo.toml`) documentent comme ayant déjà causé un panic au démarrage ("there is no reactor running") et une accessibilité AT-SPI morte.

**Confiance : à vérifier par un humain** — ne pas toucher sans tracer la résolution complète de `zbus` v4/v5 dans `Cargo.lock` et sans repro du panic potentiel.

---

## Vérifiés et jugés déjà corrects (pas de candidat)

- `reqwest` (workspace : `json, rustls-tls, cookies, stream`) — chaque sous-feature a un usage réel identifié quelque part dans le workspace (`cookie` dans `qbz-qobuz/src/client.rs`, `bytes_stream`/`wrap_stream` dans 4 crates, `.json()` dans ~10 crates). Comme Cargo unifie les features sur tout le graphe compilé ensemble, restreindre par crate n'aurait de toute façon aucun effet sur les binaires (`qbz`, `qbzd`) construits en même temps que ces crates.
- `ashpd` (`qbz-credentials` : `async-io, secret` ; `qbz-media-controls` : `async-io, notification`) — chaque feature correspond exactement à l'API portal appelée (`secret::retrieve`, `notification::NotificationProxy`).
- `keyring` (`async-secret-service, crypto-rust`) — nécessaires pour le backend Linux secret-service chiffré ; usage confirmé dans `qbz-credentials/src/lib.rs`.
- `rusqlite` (`bundled`) partout — choix délibéré de linkage statique SQLite, hors sujet allègement de features logicielles.
- `rupnp` (`full_device_spec` dans `qbz-cast`) — le crate cible des renderers UPnP/DLNA arbitraires (pas un device connu à l'avance), donc la surface complète de parsing de device description est plausiblement nécessaire. **Pas assez de certitude pour trancher** sans documentation rupnp — laissé de côté, pas listé comme candidat actif.
- `reqwest = { features = ["blocking"] }` dans `qbz-media-controls` — `reqwest::blocking::Client` bien utilisé dans `notify.rs`.
- `tokio-tungstenite` (`rustls-tls-webpki-roots`, `qconnect-transport-ws`) — cohérent avec le reste du repo qui utilise rustls partout (pas d'OpenSSL).

---

## Résumé (3-5 lignes)

Le candidat le plus net est `tokio = { features = ["full"] }` au niveau workspace, hérité par 15 crates alors que l'usage réel ne couvre qu'un sous-ensemble (`rt, rt-multi-thread, macros, sync, time, fs, net, io-util, signal`) — le code lui-même le prouve en redéclarant un `tokio` plus étroit à 4 endroits pour éviter `full`. Deuxième trouvaille, hors-sujet allègement mais notable : `qbz`'s `image` crate n'a pas la feature `webp` alors que l'UI (file dialogs) propose `.webp` à l'utilisateur — un commentaire du code documente déjà que ça échoue au runtime, donc un vrai bug de mismatch feature/UX, pas juste du poids superflu. Le seul autre point touché à l'audio (zbus `tokio` sur `qbz-audio`, usage 100% blocking) est classé "à vérifier" car un commentaire du repo indique que c'est une unification délibérée contre un panic executor déjà rencontré ailleurs — ne pas y toucher sans tracer tout le graphe zbus.
