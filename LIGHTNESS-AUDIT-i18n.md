# Audit i18n — `crates/qbz-i18n/`

Lecture seule, aucune modification, aucune compilation. Date : 2026-08-20.

## Fichiers examinés

- `crates/qbz-i18n/src/lib.rs`
- `crates/qbz-i18n/src/po.rs`
- `crates/qbz-i18n/src/plural.rs`
- `crates/qbz-i18n/Cargo.toml`
- `crates/qbz-ui/translations/*/LC_MESSAGES/qbz-ui.po`

## Combien de langues sont embarquées dans le binaire ?

**8 langues**, en dur dans `lib.rs` :

```rust
const LANGS: [&str; 8] = ["en", "es", "de", "fr", "pt", "ru", "ja", "nl"];
```

Chacune est bundlée via `include_str!` sur le `.po` correspondant :

| Langue | Fichier source | Taille sur disque |
|---|---|---|
| en | `crates/qbz-ui/translations/en/LC_MESSAGES/qbz-ui.po` | 20 K |
| es | `crates/qbz-ui/translations/es/LC_MESSAGES/qbz-ui.po` | 120 K |
| de | `crates/qbz-ui/translations/de/LC_MESSAGES/qbz-ui.po` | 120 K |
| fr | `crates/qbz-ui/translations/fr/LC_MESSAGES/qbz-ui.po` | 120 K |
| pt | `crates/qbz-ui/translations/pt/LC_MESSAGES/qbz-ui.po` | 120 K |
| ru | `crates/qbz-ui/translations/ru/LC_MESSAGES/qbz-ui.po` | 152 K |
| ja | `crates/qbz-ui/translations/ja/LC_MESSAGES/qbz-ui.po` | 136 K |
| nl | `crates/qbz-ui/translations/nl/LC_MESSAGES/qbz-ui.po` | 116 K |

**Total : ~887 Ko** (907 905 octets exacts, somme des `.po` sources).

Aucun `.mo`, aucun autre catalogue (Slint n'a pas de `@tr` catalogue séparé ici — `qbz-i18n` est le seul mécanisme, explicitement conçu "frontend-agnostic" / sans dépendance Slint, cf. commentaire ADR-006 en tête de `lib.rs`).

Ces 8 fichiers `.po` sont les **seuls** `include_str!` de type texte de traduction dans tout le dépôt (les autres `include_str!` trouvés sont des README de crates vendorées, des shaders WGSL/GLSL, et des fixtures de test JSON — sans rapport avec l'i18n).

Il n'existe **aucun feature flag Cargo** conditionnant l'inclusion d'une langue : `Cargo.toml` du crate est vide de toute section `[features]` (juste `[package]` + `[dependencies]` vide). Les 8 `.po` sont donc **toujours** compilés dans le binaire, quelle que soit la plateforme ou la configuration de build.

## Chargement : lazy ou tout au démarrage ?

**Le texte source `.po` est toujours embarqué** (8 constantes `&str` statiques, donc dans le binaire final, section rodata) — ça, c'est non négociable avec `include_str!`.

**Le *parsing* en `Catalog`, en revanche, est paresseux** — une seule langue est parsée à la fois, à la demande :

```rust
static CATALOGS: [OnceLock<Catalog>; 8] = [OnceLock::new(); 8 fois];

fn catalog(idx: u8) -> &'static Catalog {
    CATALOGS[idx].get_or_init(|| Catalog::parse(LANGS[idx], src))
}
```

- Chaque langue a son propre `OnceLock<Catalog>`.
- `catalog(idx)` ne parse que lors du premier accès à cet index (`get_or_init`).
- `current_catalog()` n'appelle que `catalog(CURRENT.load())` — donc seule la langue **active** est jamais parsée, sauf changement de langue en cours de session (auquel cas la nouvelle langue est parsée à son tour et reste en cache).
- Changer de langue (`set_language`) ne fait que changer un index atomique (`AtomicU8`) — aucun re-parsing des langues déjà vues, aucune purge des catalogues déjà parsés (donc si l'utilisateur bascule entre 3 langues dans une session, les 3 `Catalog` restent en mémoire, jamais libérés).

## Résumé mémoire

- **Disque / binaire (texte brut)** : les 8 `.po` (~887 Ko) sont **toujours** présents dans le binaire, non conditionnels, aucun moyen de les exclure au build actuel (pas de feature flag).
- **RAM (structures parsées)** : seule(s) la (les) langue(s) réellement sélectionnée(s) pendant la session sont parsées en `HashMap` — le `Catalog` d'une langue jamais activée reste à l'état `OnceLock` vide, donc coût mémoire ~0 pour les langues non utilisées.

## Constat "lightness"

Le point structurel : **8 langues × ~110 Ko en moyenne = ~887 Ko de texte `.po` cru gonflent le binaire pour tout le monde**, y compris un utilisateur mono-langue anglais, sans qu'aucun feature flag Cargo permette de les exclure sélectivement à la compilation. Le runtime est déjà optimisé côté RAM (lazy parsing par `OnceLock`), mais côté taille de binaire, rien n'évite l'embarquement des 7 langues non utilisées.
