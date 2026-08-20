# Audit réseau — `qbz-qobuz/src/client.rs` + `retry.rs`

Lecture seule, aucun fichier modifié. Portée : `crates/qbz-qobuz/src/client.rs` (3180 L) et `crates/qbz-qobuz/src/retry.rs` (198 L).

## 1. Connexion HTTP — pool réutilisé, pas de recréation par appel

`QobuzClient::new()` (client.rs:79-98) construit **un seul** `reqwest::Client` :

```rust
let http = Client::builder()
    .user_agent(USER_AGENT)
    .cookie_store(true)
    .connect_timeout(std::time::Duration::from_secs(10))
    .build()?;
```

Ce client vit dans le champ `http: Client` de la struct `QobuzClient` (ligne 51). `impl Clone for QobuzClient` (lignes 63-75) clone `self.http.clone()` — un clonage de `reqwest::Client` est un clonage d'`Arc` interne, donc bon marché et **partage le même pool de connexions** (reqwest utilise `hyper` + un pool de connexions keep-alive en interne). Tous les appels API passent soit par `self.http` directement (`get_http()`, ligne 214, exposé publiquement pour `catalog search`), soit via le choke point `self.http()` (ligne 228) qui ajoute juste la porte offline.

→ **Pas de TLS handshake répété par requête.** Le pool de connexions HTTP/1.1 ou HTTP/2 keep-alive de reqwest est réutilisé pour toute la durée de vie du `QobuzClient` (et de ses clones, qui partagent le même client interne).

Deux autres `Client::new()` détectés aux lignes 3115 et 3148 sont dans `#[cfg(test)]` (tests unitaires `offline_gate_*`) — chacun instancie un `QobuzClient` de test isolé, ce n'est pas un chemin de production. Non problématique.

**Hors périmètre mais noté en passant** : `grep -rn "Client::builder\|reqwest::Client::new"` sur tout le repo montre que d'autres crates du projet (`qbz-playlist-import/src/http.rs`, `qbz/src/log_viewer.rs`, `qbzd/src/qconnect/engine.rs`, `qbz-music-link/src/fast_path.rs`, etc.) construisent des `reqwest::Client` ad hoc à chaque appel plutôt que de réutiliser une instance partagée — `qbz-playlist-import/src/http.rs` le documente même explicitement en commentaire (« `Client::new()` per Tidal fetch »). Ce n'est **pas** un problème dans `client.rs`/`retry.rs`, mais si un futur audit élargit le périmètre, ces fichiers sont les candidats à un vrai gaspillage de handshakes TLS.

## 2. Caches internes — pas de re-fetch identifiable dans le fichier

Trois caches en mémoire, protégés par `Arc<RwLock<...>>`, évitent des appels réseau redondants :

- **`validated_secret`** (`secret()`, lignes 236-256) : le secret d'app n'est validé (`test_secret`, requête réseau) qu'une seule fois — lecture rapide en premier, écriture seulement si absent. Tout appel signé ultérieur (`search_*`, `get_file_url`, etc.) réutilise ce secret sans repasser par le réseau.
- **`cmaf_session`** (`ensure_cmaf_session()`, lignes 2898-2976) : verrouillage double-check documenté explicitement (commentaire lignes 2895-2897) — chemin rapide en lecture (`session valide si > 60 s restantes`), chemin lent en écriture qui re-vérifie sous verrou exclusif avant de POST `session/start`. Empêche des appels concurrents de démarrer chacun une session CMAF différente (bug historique documenté dans le commentaire, lié à une corruption de clé AES).
- **`tokens` (bundle)** (`init()`, lignes 149-186) : démarrage à chaud sert les tokens en cache immédiatement, rafraîchissement en tâche de fond seulement si la version du bundle a changé (`refresh_bundle_if_changed`), pas de re-téléchargement systématique du bundle (~7 Mo).

**Absence de cache notée** : les méthodes de lecture catalogue (`get_track`, `get_album` ligne 852, `get_artist` lignes 1641/1664, `get_playlist` ligne 1861, etc.) n'ont **aucune** couche de cache dans ce fichier — chaque appel retape le réseau, y compris si le même ID est redemandé juste après (ex. navigation avant/arrière dans l'UI). Ce n'est pas un bug en soi (le fichier n'a pas vocation à porter un cache HTTP), mais c'est un point d'attention si un audit plus large veut réduire le trafic redondant — la responsabilité de ne pas re-fetcher serait alors côté appelant (`qbz-core`/`qbz-app`), non visible depuis `client.rs`.

Pas de duplication identifiée entre méthodes du fichier lui-même (`search_albums`/`search_tracks`/`search_artists`/`catalog_search` sont des endpoints Qobuz distincts et légitimement séparés, pas une même info re-fetchée deux fois).

## 3. `retry.rs` — pas d'amplification de trafic

- **`DEFAULT_MAX_ATTEMPTS = 3`** (1 essai initial + 2 retries) — borné, pas de boucle illimitée.
- **Backoff exponentiel avec jitter** (`backoff_delay`, lignes 71-85) : ~250 ms, ~500 ms, ~1 s, plafonné à 2 s, + jusqu'à 25 % de jitter (dérivé de l'horloge, pas de dépendance `rand`). Pas de retry immédiat en rafale.
- **Classification stricte transient/terminal** (`classify_status`, lignes 59-66) : seuls 5xx et 429 (Too Many Requests) sont retryables ; tout le reste (404, 403, 4xx) est terminal et remonte immédiatement — pas de retry inutile sur des erreurs définitives.
- Intégration avec le **circuit breaker 403** (`forbidden_breaker`, appelé avant le réseau dans `get_file_url`/`ensure_cmaf_session`) : après une série de 403, le court-circuit empêche même la première tentative de retry de toucher le réseau, ce qui protège contre un blocage IP en cas d'incident (issue #637 citée en commentaire).
- Chaque tentative régénère un timestamp + signature frais (ligne 3006-3008, commentaire explicite) — évite de renvoyer une requête signée expirée qui échouerait de toute façon.

→ Rien dans `retry.rs` ne peut amplifier le trafic réseau de façon incontrôlée : borné, backoff croissant avec plafond, jitter, classification correcte, et coupé en amont par le breaker en cas d'incident soutenu.

## Conclusion

Aucun problème d'efficacité réseau identifié dans le périmètre audité. Le client HTTP est un singleton partagé (pool de connexions réutilisé), les caches internes (secret, session CMAF, bundle tokens) évitent les re-validations/re-démarrages redondants avec un luxe de commentaires expliquant les bugs de concurrence historiques qu'ils corrigent, et la logique de retry est disciplinée (bornée, backoff+jitter, classification stricte, couplée au circuit breaker 403). Le seul point d'attention (hors scope strict) est l'absence de cache pour les lectures catalogue (get_track/get_album/get_artist/get_playlist), qui repose entièrement sur les appelants pour éviter les re-fetches ; et la présence, ailleurs dans le repo, de plusieurs `reqwest::Client` recréés par appel — un vrai gaspillage TLS mais dans des fichiers hors périmètre de cet audit.
