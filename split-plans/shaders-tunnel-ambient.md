# split-plan: `ui/shaders/tunnel.wgsl` (185) + `ui/shaders/ambient.wgsl` (179)

Two files, one plan: the seams are the same seams, and the shared chunk that
fixes one is the chunk that fixes the other.

- `crates/qbz-ui/ui/shaders/tunnel.wgsl` — 185 lines (100 code, 60 comment, 25 blank)
- `crates/qbz-ui/ui/shaders/ambient.wgsl` — 179 lines (108 code, 50 comment, 21 blank)

---

## 0. Feasibility: how are these loaded, and can a `.wgsl` file be split at all?

Answered with the loader, not with assumption.

**Mechanism.** `crates/qbz/src/shader_underlay/resources/mod.rs:79-86`:

```rust
pub(super) const SHADER_SOURCES: &[&str] = &[
    include_str!("../../../../qbz-ui/ui/shaders/plasma.wgsl"),          // [0] mode 1
    include_str!("../../../../qbz-ui/ui/shaders/tunnel.wgsl"),          // [1] mode 2
    ...
    include_str!("../../../../qbz-ui/ui/shaders/ambient.wgsl"),         // [5] mode 7 (app-wide)
];
```

and `crates/qbz/src/shader_underlay/resources/pipelines.rs:18-21`:

```rust
source: wgpu::ShaderSource::Wgsl(SHADER_SOURCES[idx].into()),
```

`line_bed.wgsl` is loaded the same way but inline, at `pipelines.rs:59-61`.
Slint does not touch the `.wgsl` files at all — the only Slint references are
comments in `ui/state/immersive_window_control.slint:50-63` naming the scenes.
There is no build script, no preprocessor, no `#include` anywhere.

So each file is `include_str!`-ed into a `&'static str` and handed to
`wgpu::ShaderSource::Wgsl` as one translation unit, compiled by naga 28.0.0
(`crates/Cargo.lock`) at **runtime**, lazily, the first time that scene is
selected.

**Consequence.** WGSL has no import in the version naga consumes, so the file
cannot be split by the shader language. But the loader is a Rust string, and a
Rust string can be assembled from several files before it reaches naga. Option
(b) from the brief applies, and it is cheap:

```rust
concat!(include_str!("…/common/prelude.wgsl"), include_str!("…/ambient.wgsl"))
```

Verified, not assumed: `concat!` eagerly expands nested `include_str!` and
yields a `&'static str` (compiled and run standalone under this toolchain
before writing this plan). That means **`SHADER_SOURCES` stays `&[&'static str]`
and `pipelines.rs` needs no change at all** — no `String`, no allocation, no
lifetime churn. There is no shader preprocessor to invent; the concatenation is
one macro.

**Is the duplication real?** Option (c) is the deciding evidence. Hashing the
declaration blocks across all seven shaders:

| block | identical in |
|---|---|
| `struct Uniforms` (16 lines) | **all 7** — byte-identical |
| `@group(0) @binding(0) var<uniform> u` | all 7 |
| `struct VsOut` + `fn vs_main` (fullscreen triangle) | 6 of 7, identical modulo comments (`line_bed` has its own projecting vertex stage) |
| `fn hash2` (integer lattice hash, 7 lines + 14 lines of rationale comment) | `ambient` + `plasma`, byte-identical |
| `fn vnoise` | `ambient` + `plasma`, identical except one local renamed `w`/`uf` |
| `fn band_at` | `tunnel` + `liquid_spectrum`, byte-identical |

Roughly 40 lines of declaration boilerplate are copied into every scene, and the
`Uniforms` block — the one thing that must stay byte-compatible with the
`#[repr(C)]` struct at `shader_underlay/mod.rs:63-88` — currently exists in
**seven** hand-maintained copies. `mod.rs:89` already admits the hazard: *"WGSL
is compiled at runtime (naga), so cargo cannot catch a Rust/WGSL layout
mismatch."* Collapsing seven copies to one is a correctness improvement
independent of the line budget.

**Also found:** `ambient.wgsl:92-96` declares `fn blob(...)` which is never
called anywhere in the file. Dead code; delete it (`plasma`'s `blob` is a
different formula and stays local to plasma).

**Recommendation: split, via concatenated chunks.** Not a documented
exception. The mechanism already supports it, the change to Rust is one macro
per entry, and the split removes duplication that exists today rather than
chopping a coherent file into arbitrary halves.

**Honest limitation, stated up front:** this satisfies the per-file 130-line
rule and deletes real duplication. It does **not** make the compiled shader
smaller — the concatenated translation unit naga sees is still ~185 lines. If
the rule is read as "no translation unit over 130 lines", these shaders cannot
satisfy it and should take an exception instead. This plan reads it as a
per-file rule, which is how it is worded.

---

## 1. Why are these files long?

Neither is one irreducible thing.

**`tunnel.wgsl`** is four responsibilities stacked in one file:
1. lines 17-54 — the shared scene prelude (Uniforms, binding 0, VsOut, vs_main). Not tunnel's; copied.
2. lines 56-63 — `band_at`, the 8-band FFT selector. Not tunnel's; shared with `liquid_spectrum`.
3. lines 65-75 — three tunables (`FLIGHT_SPEED`, `BEND_AMT`, `SPOKES`) with the wrap-arithmetic rationale.
4. lines 77-185 — the fragment, which is itself two phases: **path geometry** (aspect/scale, vanishing-point sway, box distance, depth, the winding centerline, ring index) at 94-133, and **shading** (frames, corner lines, speed-lines, palette, beat punch) at 135-185. The two phases communicate through four values.

**`ambient.wgsl`** is three:
1. lines 14-50 — the same shared prelude.
2. lines 52-90 — `hash2` / `vnoise` / `fbm`, generic noise, duplicated in `plasma`, carrying a 14-line comment on why the hash is integer-based. Not ambient-specific.
3. lines 92-96 — `blob`, dead.
4. lines 98-179 — `mball` + the fragment: the actual ambient scene.

In both files the scene-specific part is well under budget. The overflow is
boilerplate and shared helpers.

---

## 2. Seams — exact ranges that move

### New chunk files (`crates/qbz-ui/ui/shaders/common/`)

| file | source | est. lines |
|---|---|---|
| `common/prelude.wgsl` | `tunnel.wgsl:17-54` (identical to `ambient.wgsl:14-50`) + a header explaining the chunk contract | ~50 |
| `common/noise.wgsl` | `ambient.wgsl:52-90` — `hash2`, `vnoise`, `fbm`, keeping the integer-hash rationale comment verbatim | ~43 |
| `common/fft.wgsl` | `tunnel.wgsl:56-63` — `band_at` + header | ~15 |

### New tunnel chunk (`crates/qbz-ui/ui/shaders/`)

| file | source | est. lines |
|---|---|---|
| `tunnel_path.wgsl` | `tunnel.wgsl:65-75` (tunables) + `tunnel.wgsl:94-133` refactored into `struct Corridor { p: vec2<f32>, r: f32, ring_id: i32, ring_frac: f32 }` and `fn corridor(uv: vec2<f32>) -> Corridor` | ~70 |

`corridor()` takes no uniform arguments — `u` is module scope, so it reads
`u.resolution`, `u.time`, `u.phase`, `u.level_smooth`, `u.beat` directly, exactly
as the inline code does today. `flight` is consumed entirely inside `corridor()`
and does not cross the seam.

### Resulting target files

| file | before | after | contents |
|---|---|---|---|
| `tunnel.wgsl` | 185 | **~90** | its header comment (1-15), the FFT aggregates (81-92), one `corridor()` call + four `let` destructures, and the shading block (135-185) |
| `ambient.wgsl` | 179 | **~96** | its header comment (1-12), `mball` (98-104), the fragment (106-179) |

Arithmetic: tunnel 185 − 39 (prelude+blank) − 9 (`band_at`+blank) − 12
(tunables) − ~40 (geometry) + ~9 (call/destructure/comment) ≈ 90.
Ambient 179 − 38 (prelude) − 39 (noise) − 6 (dead `blob`) ≈ 96.

Every new file is under 130.

---

## 3. Public surface — exact concatenation order

```
tunnel  = prelude.wgsl + fft.wgsl + tunnel_path.wgsl + tunnel.wgsl
ambient = prelude.wgsl + noise.wgsl + ambient.wgsl
```

Declaration-before-use in that order: `Uniforms` and `u` (prelude) before
`band_at`/`hash2` which read `u`; `Corridor`/`corridor` before the `fs_main`
that calls it. WGSL module scope is order-independent per spec and naga 28
implements that, but **do not rely on it** — order the chunks
declaration-before-use so the concatenated text is valid under any stricter
frontend and so a human reading the concatenation reads it top-down.

**Chunk contract, which every adopting scene must honour:**

- Exactly one `struct Uniforms`, one `@group(0) @binding(0)`, one `struct VsOut`,
  one `fn vs_main`, one `fn fs_main` in the whole concatenation. A scene adopting
  `prelude.wgsl` **must delete its own copies** of the first four, or naga fails
  with a duplicate-declaration error at pipeline build.
- The prelude owns the vertex stage; the scene owns `fs_main`. `pipelines.rs`
  keeps asking for entry points `vs_main` / `fs_main` — unchanged.
- Chunk files are **not standalone shaders**. Name the directory `common/` and
  say so in the header of each chunk, so nobody adds `common/noise.wgsl` to
  `SHADER_SOURCES` as if it were a scene.
- A scene may declare additional bindings (plasma has 1 and 2, spectral_ribbon 2
  and 3) in its own file; the prelude declares only binding 0. A shader using a
  subset of the bind-group layout is valid — already relied upon
  (`ambient.wgsl:10-11`, `resources/mod.rs:56-62`).

### Rust changes

`crates/qbz/src/shader_underlay/resources/mod.rs` — two entries in
`SHADER_SOURCES`:

```rust
include_str!("../../../../qbz-ui/ui/shaders/plasma.wgsl"),              // [0] mode 1
concat!(                                                                // [1] mode 2
    include_str!("../../../../qbz-ui/ui/shaders/common/prelude.wgsl"),
    include_str!("../../../../qbz-ui/ui/shaders/common/fft.wgsl"),
    include_str!("../../../../qbz-ui/ui/shaders/tunnel_path.wgsl"),
    include_str!("../../../../qbz-ui/ui/shaders/tunnel.wgsl"),
),
...
concat!(                                                                // [5] mode 7
    include_str!("../../../../qbz-ui/ui/shaders/common/prelude.wgsl"),
    include_str!("../../../../qbz-ui/ui/shaders/common/noise.wgsl"),
    include_str!("../../../../qbz-ui/ui/shaders/ambient.wgsl"),
),
```

Plus: update the doc comment at `mod.rs:76-79` — "Adding a scene = one
`include_str!` here" becomes "one entry, either a single `include_str!` or a
`concat!` of prelude chunks + the scene", and drop "All must declare the SAME
`Uniforms` block" in favour of "all must include `common/prelude.wgsl`, which
declares it once".

`mod.rs` is 86 lines today; this adds ~10. Still under budget.

**No other Rust changes.** `pipelines.rs` is untouched: `concat!` produces a
`&'static str`, so `SHADER_SOURCES` keeps its type and `SHADER_SOURCES[idx].into()`
still works. `bgl.rs`, `shared.rs`, `sized.rs`, `render/`, `reshape.rs` — untouched.

### Files added, for the record

```
crates/qbz-ui/ui/shaders/common/prelude.wgsl   ~50
crates/qbz-ui/ui/shaders/common/noise.wgsl     ~43
crates/qbz-ui/ui/shaders/common/fft.wgsl       ~15
crates/qbz-ui/ui/shaders/tunnel_path.wgsl      ~70
crates/qbz-ui/ui/shaders/README.md             (why each chunk exists; the
                                                concatenation table above)
```

### Staging

Do the two target scenes first (step 1). Wiring the other four fullscreen
scenes to `common/prelude.wgsl` — and `plasma` to `common/noise.wgsl`,
`liquid_spectrum` to `common/fft.wgsl` — is mechanical and deletes another
~150 duplicated lines, but it touches shaders that are currently under budget
and is not required by the rule. Keep it as a separate step 2 so a regression
in step 2 cannot be confused with a regression in step 1. `line_bed.wgsl` never
adopts the prelude: its vertex stage differs and it declares binding 4.

---

## 4. What can break

Shader-specific, not the usual Slint/Rust list.

- **Duplicate declarations.** A scene that adopts the prelude but keeps its own
  `struct Uniforms` fails to compile. Symptom: that one scene renders nothing.
- **Missing declarations.** A scene that adopts nothing but had its prelude
  deleted fails the same way.
- **`corridor()` refactor drift.** Moving `tunnel.wgsl:94-133` into a function is
  the only place where shader *behaviour* can change. The values crossing the
  seam are exactly `p`, `r`, `ringId`, `ringFrac`; `depth`, `z`, `z0`, `depth0`,
  `flight`, `bend`, `wind*` are internal. Anything else silently read downstream
  is a bug. `depthShade`, `nearWeight`, `palT` and `wallLit` (lines 159-168) all
  derive from `p` and `r` only — verify that before moving.
- **Comment-only differences.** `vs_main` bodies are identical across the six
  scenes only *modulo comments*. The prelude keeps one version; the comments in
  the others are discarded. Read them before deleting — `tunnel.wgsl:43` and
  `plasma.wgsl` carry slightly different notes.
- **`plasma`'s `vnoise` local rename.** If step 2 happens, `plasma` loses its
  `uf`-named local for the prelude's `w`. Same arithmetic; no behaviour change.

---

## 5. Risks

**The top risk is that WGSL errors are runtime errors, not compile-time errors.**
`cargo build` does not parse `.wgsl` at all. `include_str!` will happily embed a
syntactically broken shader, the crate links clean, and the failure appears only
when naga compiles the module — inside `build_scene_pipeline`, which is **lazy
and per-scene** (`resources/mod.rs:56-66`: "each compiled+cached on its first
use"). So a broken tunnel shader is invisible until a user selects immersive
mode 2, and a broken ambient shader is invisible until the app-wide dynamic
background (mode 7) is switched on. A green build and a green test run prove
nothing about these files.

Mitigations, in order of value:

1. **Validate the concatenated sources in a test with naga.** Add
   `naga = "=28.0.0"` as a dev-dependency (same version wgpu 28 already pulls, so
   it dedupes) and a `#[cfg(test)]` test that runs
   `naga::front::wgsl::parse_str` + `naga::valid::Validator` over every entry of
   `SHADER_SOURCES` and over `line_bed.wgsl`. This is the single change that
   converts the whole risk class from runtime to test time, and it is worth doing
   whether or not the split happens.
2. **Caveat on (1): CI does not run it as-is.** `scripts/cargo-test.sh` and
   `.github/workflows/test-crates.yml` run the workspace with
   `--exclude qbz --exclude qbz-ui` (the Slint compile graph blows the memory
   budget). A test living in `crates/qbz` is therefore *not* covered by CI and
   must be run by hand (`cargo test -p qbz --lib shader`). To get CI coverage,
   move `SHADER_SOURCES` and the validation test into a small
   `qbz-shaders` crate with no Slint dependency — pure data plus the naga check —
   and have `qbz` re-export it. That is the right home for it under the
   pure/IO/render separation anyway, but it is a bigger change than this split
   and should be decided separately.
3. **A cheap textual guard, if (1) is declined.** A test asserting each
   `SHADER_SOURCES` entry contains exactly one `fn vs_main`, one `fn fs_main` and
   one `struct Uniforms`. It catches the most likely concatenation mistake
   (duplicate or missing prelude) without any new dependency. Same CI caveat.
4. **Manual smoke, mandatory before merging.** Launch the app, cycle immersive
   mode to 2 (Tunnel), then enable the app-wide dynamic background (mode 7,
   Ambient), and watch the log for `[shader] scene pipeline {idx} built`
   (`pipelines.rs:47`). A pipeline that fails to compile is the loud case; a
   pipeline that compiles but renders differently is the quiet one, so compare
   tunnel against a pre-change build side by side — the `corridor()` refactor is
   exactly the kind of change that survives compilation and changes pixels.

Secondary risks:

- **Error line numbers stop matching files.** naga reports offsets into the
  concatenated string, so "line 142" is a line in a source that exists nowhere on
  disk. Mitigate by logging the chunk list and each chunk's line count when
  `create_shader_module` fails, or by dumping the concatenated source to a temp
  file behind a debug env var. Without this, debugging a shader error gets
  meaningfully worse — this is the real cost of the approach and should be paid
  up front, not after the first confusing error.
- **A chunk edited for one scene breaks another.** `common/prelude.wgsl` is on
  the compile path of two scenes after step 1 and six after step 2. Editing it to
  suit one scene silently changes all of them. The naga test in (1) covers
  compilation; it does not cover appearance.
- **The `Uniforms` drift guard weakens in the wrong direction if this is done
  carelessly.** Today the static assert in `shader_underlay/mod.rs:89` guards the
  Rust side against seven WGSL copies. After the split there is one copy, which
  is strictly better — but only if every scene actually adopts the prelude rather
  than keeping a stale local copy that still compiles fine on its own. The
  "exactly one `struct Uniforms`" assertion in (3) is what keeps that honest.
