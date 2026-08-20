# Audit — chargement Immersive / Kiosk au démarrage

Contexte : l'appli est perçue comme "lourde". Hypothèse à vérifier : les
overlays spectraux WGSL du mode Immersive (`crates/qbz-ui/ui/immersive/`,
13 fichiers) et les vues quasi dupliquées du mode Kiosk
(`crates/qbz-ui/ui/shell/Kiosk*.slint`) sont-ils instanciés/chargés dès le
démarrage de l'appli, même quand la session ne les utilise jamais ?

**Conclusion : non.** Les trois couches vérifiées (arbre Slint, pipeline
WGSL/wgpu, tap FFT) sont déjà gardées par des conditions paresseuses, du
compilateur Slint jusqu'au code Rust. Aucun correctif de sécurité n'a été
appliqué — rien d'imprudent trouvé à corriger.

## 1. Le `if` Slint instancie-t-il paresseusement ?

Oui, confirmé à la source (pas de supposition) :

- Doc officielle Slint : *"The `if` construct instantiates an element only
  if a given condition is true."*
- Compilateur vendored en local
  (`~/.cargo/registry/src/.../i-slint-compiler-1.16.1/object_tree.rs`,
  `llr/lower_to_item_tree.rs`) : un élément `if` est lowered comme un
  `RepeatedElementInfo` avec `is_conditional_element: true` — **le même
  mécanisme runtime qu'un `for`** (un `Repeater<T>` sur un modèle à 0 ou 1
  élément), juste sans les props d'index/data. Le sous-arbre d'un `if` faux
  n'est ni construit, ni ses bindings évalués, tant que la condition ne
  passe pas à vrai.

Conséquence : chaque écran top-level d'`app.slint` est mutuellement
exclusif —

```
if root.screen == AppScreen.splash: SplashScreen { }
if root.screen == AppScreen.login:  LoginScreen { ... }
if root.screen == AppScreen.shell:  AppShell { ... }     // desktop
if root.screen == AppScreen.kiosk:  KioskShell { ... }   // kiosk
```

`KioskShell` (et donc les 12 fichiers `Kiosk*.slint` qu'il importe) n'est
instancié que si `root.screen == AppScreen.kiosk`. Une session desktop
normale ne construit jamais cet arbre. Le profil (`kiosk_profile_active()`,
`crates/qbz/src/main.rs:7669`) ne fait que choisir la VALEUR initiale de
`screen` — un simple enum, aucun coût de construction.

De même dans `AppShell.slint` (ligne 933) :

```
if ImmersiveState.open: ImmersiveView { ... }
```

`ImmersiveView` et tout son sous-arbre (`CoverflowPanel`,
`ImmersiveWaveBedPanel`, `ImmersiveSpectralOverlay`, etc.) ne sont montés
qu'à l'ouverture réelle du mode Immersive.

## 2. Les shaders WGSL sont-ils compilés au démarrage ?

Non. Le pipeline est en trois temps, et seul le premier temps (gratuit) se
produit au démarrage :

1. **`shader_underlay::setup()`** (`crates/qbz/src/shader_underlay.rs:349`)
   — appelé une fois à `RenderingState::RenderingSetup` (démarrage du
   renderer wgpu, donc démarrage de l'appli sur le tier fort). Le
   commentaire du code le dit explicitement : *"Deliberately CHEAP: no WGSL
   compilation, no texture allocation — those happen lazily in
   `render_frame` on first shader use, so sessions that never open a
   shader scene pay nothing at window paint."* Il ne fait que stasher le
   `Device`/`Queue`.
2. **Compilation WGSL + pipelines** (`build_scene_pipeline`,
   `build_linebed_pipeline`, lignes 649–727) — appelée depuis
   `render_frame()` (ligne 764), UNE SEULE FOIS par mode, la première fois
   que ce mode précis est réellement rendu (`res.pipelines[idx].is_none()`
   / `res.linebed_pipeline.is_none()` gate chaque `create_shader_module` +
   `create_render_pipeline` individuellement). Les 6 shaders
   (`plasma`, `tunnel`, `aurora`, `spectral_ribbon`, `liquid_spectrum`,
   `ambient`) sont donc compilés indépendamment, à la demande.
3. **`render_frame()` lui-même** n'est appelé par le drain 30 fps
   (`crates/qbz/src/visualizer.rs`) que lorsque le timer tourne, et ce
   timer est démarré/arrêté par le handler `on_set_enabled`
   (`VisualizerState.set-enabled`) piloté depuis `AppShell.slint` (ligne
   166–177) par :
   `ImmersiveState.open || (Large dock actif) || (background mode "Ambient" + piste en cours)`.
   Une session qui n'ouvre jamais Immersive et n'active jamais le fond
   d'écran animé ne déclenche donc jamais `render_frame`, donc jamais la
   compilation d'aucun `.wgsl`.

Le code source `.wgsl` (`ui/shaders/*.wgsl`) est bien embarqué dans le
binaire via `include_str!` à la compilation Rust (normal, coût disque nul à
l'exécution) — ce n'est pas la même chose que la compilation GPU
(`create_shader_module`), qui elle est paresseuse.

## 3. Autres points vérifiés (rien à signaler)

- Les singletons Slint `ImmersiveState` / `VisualizerState` (globals,
  toujours instanciés) n'ont que des valeurs par défaut triviales (bool,
  couleur, image vide) — aucun chargement d'asset au niveau global.
- Le tap FFT (`visualizer::install`, appelé une fois au démarrage) est
  "Inert (tap disabled, no capture / no FFT cost) until the immersive view
  opens" selon son propre commentaire (`crates/qbz/src/main.rs:8648-8652`)
  — confirmé par le gate `on_set_enabled` ci-dessus.
- `queue.rs::row_from` (modèle plat "coverflow") est recalculé à chaque
  changement de queue, QUE le mode Immersive soit ouvert ou non — mais ne
  fait que cloner des `String` (id, titre, artiste, URL d'artwork), aucun
  décodage d'image ni I/O. Coût négligeable, pas de "chargement d'asset" au
  sens de la tâche ; je ne l'ai pas touché.

## Verdict

Aucun correctif appliqué. L'architecture existante gate déjà correctement
les deux modes à trois niveaux indépendants (arbre Slint conditionnel,
compilation GPU différée, timer de drain conditionnel) — le code documente
lui-même ces choix ("Deliberately CHEAP", "Inert … until"). La lourdeur
perçue de l'appli n'a donc probablement pas sa source dans un chargement
prématuré des composants Immersive/Kiosk ; elle est ailleurs (pistes non
explorées ici : taille du binaire/temps de démarrage du renderer lui-même,
nombre de globals Slint évalués au boot, autres vues toujours montées dans
`AppShell` hors Immersive/Kiosk).
