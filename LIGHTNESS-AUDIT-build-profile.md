# Audit — profil de build release (taille du binaire)

Lecture seule. Portée : `crates/Cargo.toml` `[profile.release]` + `.github/workflows/release-linux.yml`.

## État actuel

`crates/Cargo.toml` (lignes 94-104) :
```
[profile.release]
strip = "symbols"
```
Seul levier déclaré dans le manifest. Mesuré 2026-07-02 : 400 MB → 218 MB (-45%), dû à `.strtab`/`.symtab` gonflés par les noms générés par Slint. Le commentaire précise explicitement que les "codegen knobs" (opt-level / codegen-units / jobs) restent volontairement dans les workflows (`CARGO_PROFILE_RELEASE_*` en env), pas dans le manifest.

## Ce qui est déjà appliqué en CI (invisible dans Cargo.toml)

`.github/workflows/release-linux.yml`, job `build` (ligne ~151-156), commenté "MIN memory tier" :
```
CARGO_INCREMENTAL: "0"
CARGO_BUILD_JOBS: "1"
CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "256"
CARGO_PROFILE_RELEASE_OPT_LEVEL: "2"
CARGO_PROFILE_RELEASE_STRIP: "none"   # override — restripe manuellement après coup
RUSTFLAGS: "-C link-arg=-fuse-ld=mold -Z threads=1"
```
Important : `codegen-units = 256` n'est PAS un levier de réduction de taille déjà pris — c'est l'inverse (le défaut Rust est 16, ici poussé à 256 pour limiter la mémoire du runner CI). `opt-level = 2` (au lieu de 3, le défaut release) est le même compromis : mémoire de build avant taille/perf. `STRIP=none` pendant la compilation, puis un `strip -o` manuel après coup pour produire à la fois le binaire livré (strippé) et une copie non strippée archivée 90 jours pour symbolication.

Donc : ces réglages CI existent pour tenir dans la mémoire du runner (le job s'appelle "MIN memory tier", avec 18G de swap ajouté juste avant), pas pour la taille du binaire. Ils ne recoupent aucun des leviers demandés.

## Leviers non utilisés

| Levier | État | Effet attendu | Coût |
|---|---|---|---|
| `lto` | absent partout (ni Cargo.toml ni workflow) | réduction de taille + perf, souvent la plus grosse gagne après le strip symbols | compile nettement plus lent, pression mémoire accrue — à contre-courant du tiering "MIN memory" déjà en place en CI |
| `codegen-units = 1` | absent ; CI force au contraire `256` (voir ci-dessus) | réduction de taille, meilleure inlining | compile plus lent — idem, va à l'encontre du choix CI actuel de rester en 256 pour tenir en mémoire |
| `panic = "abort"` | absent | réduit la taille en supprimant les tables de déroulement de pile (landing pads / personality) | **voir risque ci-dessous — bloquant** |

## Risque `panic = "abort"` — usage réel de `catch_unwind`

`crates/qbz-player/src/player/mod.rs`, fonction `decode_with_fallback` (lignes 354-402) :
- ligne 362 : `panic::catch_unwind(AssertUnwindSafe(|| Decoder::new(...)))` — tente le décodage audio "primaire" (rodio), et si ça panique, log un warning et tente un fallback mp4.
- ligne 378 : même pattern pour la tentative `Decoder::new_mp4`, avec fallback supplémentaire vers `decode_with_symphonia` si ça panique aussi.

C'est un mécanisme de résilience de décodage (pas juste un crash-sentinel de démarrage) : la fonction essaie successivement 3 chemins de décodage (rodio → mp4 → symphonia) et `catch_unwind` sert précisément à absorber un panic du decoder tiers pour retomber sur le chemin suivant, plutôt que de faire crasher le lecteur audio à chaque piste mal formée.

`panic = "abort"` désactive `catch_unwind` (comportement non défini / `abort()` immédiat selon le profil) — appliquer ce levier casserait silencieusement ce fallback : un decoder qui panique ferait planter tout le process au lieu de basculer sur le format suivant. **Ne pas l'appliquer sans traiter ce site en premier** (soit l'accepter comme régression assumée, soit isoler le décodage dans un sous-process/thread avec un autre mécanisme d'isolement avant de couper panic=unwind globalement).

Aucun autre usage de `catch_unwind` / `panic::catch` / `set_hook` trouvé dans le repo (grep sur `crates/**/*.rs`), donc pas de crash-sentinel de démarrage distinct dans `main.rs` (`qbz/src/main.rs`, `qbzd/src/main.rs`) — le seul point d'accroche est ce fallback de décodage.

## Résumé

- `lto = true` et `codegen-units = 1` : leviers standards non pris, gain de taille probable, mais coût compile/mémoire qui rentre en tension avec le "MIN memory tier" déjà mis en place exprès en CI (18G de swap, jobs=1, codegen-units poussé à 256 pour l'inverse) — à tester avec un run CI dédié avant d'y toucher, pas à ajouter au manifest sans vérifier le budget mémoire du runner.
- `panic = "abort"` : gain de taille attendu, mais **bloquant en l'état** — `qbz-player/src/player/mod.rs::decode_with_fallback` dépend de `catch_unwind` pour son fallback de décodage audio (rodio → mp4 → symphonia). Ne pas recommander tel quel.
- Les réglages déjà en CI (`codegen-units=256`, `opt-level=2`, `jobs=1`) sont des contraintes mémoire de runner, pas des optimisations de taille — ne pas les confondre avec des leviers "déjà pris".
