# split-plan: `ui/shaders/plasma.wgsl` (153 L) and `ui/shaders/line_bed.wgsl` (144 L)

Two WGSL files over the 130-line budget by 23 and 14 lines.
Covered together because they share the same loading mechanism and the
same shared-chunk candidate. `tunnel.wgsl` and `ambient.wgsl` are also
over budget and are handled in a separate plan; this plan only reads them
as evidence of duplication.

---

## 0. Feasibility — how these files are loaded

Both are `include_str!`'d into Rust at compile time and handed to wgpu as
a single WGSL source string at runtime.

* `crates/qbz/src/shader_underlay/resources/mod.rs:79-86`

  ```rust
  pub(super) const SHADER_SOURCES: &[&str] = &[
      include_str!("../../../../qbz-ui/ui/shaders/plasma.wgsl"),          // [0] mode 1
      ... tunnel, aurora, spectral_ribbon, liquid_spectrum, ambient
  ];
  ```

* `crates/qbz/src/shader_underlay/resources/pipelines.rs:18-21`

  ```rust
  source: wgpu::ShaderSource::Wgsl(SHADER_SOURCES[idx].into()),
  ```

* `line_bed.wgsl` is not in `SHADER_SOURCES`; it has its own pipeline
  builder with its own `include_str!` at
  `crates/qbz/src/shader_underlay/resources/pipelines.rs:57-62`
  (line-strip topology + alpha blending, hence the separate pipeline).

Nothing else loads them. The only other `wgsl` hits under `crates/` are
vendored femtovg/Slint code and prose references in
`crates/qbz-ui/ui/state/immersive_window_control.slint:46-63` (comments
naming the scene files for the mode picker) — no Slint-side loading of
shader text at all.

**Consequence for splitting.** WGSL as consumed by naga 28 (`naga 28.0.0`,
`wgpu 28.0.0` in `crates/Cargo.lock`) has no `#include` and no module
import. A `.wgsl` file cannot reference another `.wgsl` file. So route
(a)-vs-(b/c) hinges entirely on whether the *loader* can concatenate.

It can, and at zero cost: `include_str!` expands to a string literal, so

```rust
concat!(
    include_str!(".../common/uniforms.wgsl"),
    include_str!(".../plasma.wgsl"),
)
```

is const-evaluated. `SHADER_SOURCES` stays `&[&str]`, `ShaderSource::Wgsl`
still gets a `Cow::Borrowed(&'static str)`, there is no build script, no
runtime allocation, no I/O, and cargo still tracks every chunk for
rebuilds (each `include_str!` emits its own rerun dependency). **Route (b)
is available and its mechanism is three lines of `concat!`.**

Route (c) is available *on top of* it, and this is the part that makes the
split worth doing at all rather than a line-count exercise — the
duplication is real and measured:

| chunk | byte-identical in | evidence |
|---|---|---|
| `struct Uniforms { … };` (16 L) | **all 7** shaders | md5 of the block extracted from each file is `37a939011444569b…` in all 7 |
| `struct VsOut` + `@vertex fn vs_main` fullscreen triangle (19 L) | **5 of 7** — plasma, ambient, aurora, liquid_spectrum, spectral_ribbon (line_bed and tunnel have their own) | md5 `6e3da892…` / `f61d1504…` |
| `fn hash2` (13 L incl. its 6-line rationale comment) | plasma + ambient | md5 `9f047476bf59485518ec6bf4f25e81dd` in both |
| `fn vnoise` (10 L) | plasma + ambient | identical except the local `uf` vs `w` |

`fn blob` is *not* shared: plasma's takes a `radius` and guards with
`max(r*r, 1e-4)`, ambient's uses `distance()` with no guard. Different
functions with the same name — do not merge them.

`resources/mod.rs:76-78` already documents the Uniforms block as a
hand-maintained cross-file invariant ("All must declare the SAME
`Uniforms` block … as plasma.wgsl"). Seven hand-synchronised copies of a
struct whose layout must also match a `#[repr(C)]` Rust struct
(`shader_underlay/mod.rs:63-93`, guarded only by
`const _: () = assert!(size_of::<Uniforms>() == 144)`) is the actual
defect here. The line budget is the occasion, not the reason.

**Recommendation: split, via route (c) implemented with route (b)'s
`concat!`.** An exception is not warranted, because the mechanism costs
three `concat!` calls and it deletes a documented manual-sync hazard.

### Declaration order

WGSL module-scope declarations are order-independent (the scope of a
module-scope declaration is the whole program) and naga has supported
out-of-order module scope since well before v28. Ordering is therefore not
a correctness requirement — but the plan still puts shared chunks first,
so the concatenated source reads top-down and so naga's byte offsets fall
in a predictable region. Do not rely on out-of-order resolution
deliberately; keep declarations before use.

---

## 1. Why are these files long?

**`plasma.wgsl` (153 L).** Not one irreducible thing. Four distinct
layers: (i) the boilerplate preamble shared with 4 other scenes
(lines 15-53), (ii) a general-purpose 2D value-noise kit shared with
ambient (55-78), (iii) plasma-specific field helpers `curl_noise` /
`blob` (80-92), (iv) the actual effect, `fs_main` (94-153, 60 L — the one
genuinely long unit, and it is one coherent per-pixel pipeline that should
not be cut).

**`line_bed.wgsl` (144 L).** Also multi-layer, but only the Uniforms block
is shared with anything: (i) Uniforms (15-30), (ii) line_bed's own
`VsOut` + camera/lattice constants (35-59), (iii) height-texture sampling
and B-spline interpolation, `height_at` + `curve_height` (61-84 — pure
data reconstruction, no camera, no colour), (iv) `vs_main` projection
(86-129), (v) `fs_main` shading (131-144).

Neither file is irreducible.

---

## 2. Seams and target files

New directory `crates/qbz-ui/ui/shaders/common/` for chunks that are not
standalone shader modules. These files are **fragments, not compilable
WGSL on their own** — that must be stated in each one's header comment so
nobody tries to add them to `SHADER_SOURCES`.

### New shared chunks

| file | contents (source of truth) | est. L |
|---|---|---|
| `common/uniforms.wgsl` | header comment (the std140 / `#[repr(C)]` 144-byte contract, moved from `resources/mod.rs:76-78` and `plasma.wgsl:12`) + `struct Uniforms` + `@group(0) @binding(0) var<uniform> u: Uniforms;` — from `plasma.wgsl:15-32` | ~26 |
| `common/fullscreen_vs.wgsl` | header + `struct VsOut` + `@vertex fn vs_main` — from `plasma.wgsl:36-53` | ~24 |
| `common/noise2d.wgsl` | header + `fn hash2` (with its existing 6-line bit-exactness rationale, `plasma.wgsl:55-67`) + `fn vnoise` (`69-78`) | ~28 |

`common/noise2d.wgsl` and `common/fullscreen_vs.wgsl` are consumed by
other scenes too; wiring ambient/aurora/liquid_spectrum/spectral_ribbon to
them is **out of scope for this plan** (it belongs to the tunnel/ambient
plan and to a follow-up). This plan only requires that the chunks be
authored from plasma's copies, which are byte-identical to ambient's for
`hash2` and to four files' for the preamble.

### `plasma.wgsl` after the split

| line range removed | goes to |
|---|---|
| 15-32 | `common/uniforms.wgsl` |
| 36-53 | `common/fullscreen_vs.wgsl` |
| 55-78 | `common/noise2d.wgsl` |

Remaining in `plasma.wgsl`: header comment 1-13, the plasma-only bindings
`@group(0) @binding(1) var prev_tex` / `@binding(2) var prev_samp`
(33-34), `curl_noise` (80-86), `blob` (88-92), `fs_main` (94-153).

**≈ 91 lines.**

Concatenation order: `uniforms` → `fullscreen_vs` → `noise2d` → `plasma`.

### `line_bed.wgsl` after the split

| line range removed | goes to |
|---|---|
| 15-31 | `common/uniforms.wgsl` (shared) |
| 41-85 | `line_bed_terrain.wgsl` (new, line_bed-only) |

`line_bed_terrain.wgsl` = the camera/lattice constants (`LINE_LENGTH` …
`VCENTER`, 41-59, including the comments that explain `SUBDIV` must match
`LINEBED_SUBDIV` in Rust) + `height_at` (61-64) + `curve_height` (66-84).
That is a real seam: constants + height-field reconstruction, with no
knowledge of projection or colour. **≈ 50 L** with a header.

Remaining in `line_bed.wgsl`: header 1-13, binding 33 (`heights_tex`),
`VsOut` 35-39, `vs_main` 86-129, `fs_main` 131-144.

**≈ 82 lines.**

Concatenation order: `uniforms` → `line_bed_terrain` → `line_bed`.

Note `line_bed.wgsl` must keep the `@group(0) @binding(4) var heights_tex`
declaration, because `common/uniforms.wgsl` supplies only binding 0.
`line_bed_terrain.wgsl`'s `height_at` reads `heights_tex`, which is
declared in a later chunk — legal under WGSL's module-scope rules, but if
you prefer strict declare-before-use, move the `binding(4)` line into
`line_bed_terrain.wgsl` and take it out of `line_bed.wgsl`. Recommended:
put it in `line_bed_terrain.wgsl` next to its only consumer.

Minimal fallback if the reviewer rejects the terrain chunk: sharing only
`common/uniforms.wgsl` puts line_bed at **127 L** — under budget, but with
3 lines of headroom and no other benefit. Prefer the full split.

---

## 3. Public surface

There is no importer to preserve on the WGSL side; the "public surface" is
the two Rust call sites.

**`crates/qbz/src/shader_underlay/resources/mod.rs`** — `SHADER_SOURCES`
keeps its type, index order, and `pub(super)` visibility. Only entry `[0]`
changes in this plan:

```rust
pub(super) const SHADER_SOURCES: &[&str] = &[
    concat!(
        include_str!("../../../../qbz-ui/ui/shaders/common/uniforms.wgsl"),
        include_str!("../../../../qbz-ui/ui/shaders/common/fullscreen_vs.wgsl"),
        include_str!("../../../../qbz-ui/ui/shaders/common/noise2d.wgsl"),
        include_str!("../../../../qbz-ui/ui/shaders/plasma.wgsl"),
    ),                                                                  // [0] mode 1
    ... unchanged
];
```

`concat!` requires literal arguments, so the chunks must be spelled as
`include_str!` invocations inside it — a `const` alias will not compile
there. If the repetition is objectionable, wrap it in a small local
`macro_rules! scene { ($body:literal) => { concat!(include_str!(...uniforms), include_str!(...fullscreen_vs), include_str!($body)) } }`; note the macro must take the *path literal*, not a const.

Whichever form is used, `mod.rs` must stay ≤130 L; the file is currently
86 L, so a macro plus the array fits, but check after editing.

**`crates/qbz/src/shader_underlay/resources/pipelines.rs:57-62`** —
`build_linebed_pipeline`'s inline `include_str!` becomes:

```rust
source: wgpu::ShaderSource::Wgsl(
    concat!(
        include_str!("../../../../qbz-ui/ui/shaders/common/uniforms.wgsl"),
        include_str!("../../../../qbz-ui/ui/shaders/line_bed_terrain.wgsl"),
        include_str!("../../../../qbz-ui/ui/shaders/line_bed.wgsl"),
    ).into(),
),
```

`pipelines.rs` is 91 L; this adds ~4. Still under budget.

**Doc comments to update**, or they become wrong:
* `resources/mod.rs:76-78` — "Adding a scene = one `include_str!` here"
  becomes "one `concat!(common…, include_str!(scene))` here"; the "All
  must declare the SAME `Uniforms` block" sentence is superseded by
  `common/uniforms.wgsl` and should say so.
* `resources/pipelines.rs:10-12` — "only the fragment shader source
  differs" is already loose; note that scenes are now assembled from
  chunks.
* `shader_underlay/mod.rs:63-65` — "Mirrors the WGSL `Uniforms` struct in
  all three `ui/shaders/*.wgsl`" is already stale (there are seven);
  repoint it at `common/uniforms.wgsl` as the single WGSL-side definition.
* `shader_underlay/mod.rs:60` and the `SUBDIV`/`LINEBED_SUBDIV` comment
  move with the constants into `line_bed_terrain.wgsl`.
* `crates/qbz-ui/ui/state/immersive_window_control.slint:50-63` names
  `ui/shaders/plasma.wgsl` etc. — those paths stay valid; no edit needed.

**READMEs** (project rule 3): add
`crates/qbz-ui/ui/shaders/README.md` explaining why `common/` exists, that
its files are fragments and not standalone modules, and the required
concatenation order per scene.

---

## 4. Tests (project rule 2)

The whole point of the exercise is undermined if a bad concatenation is
only discovered when a user picks mode 1. Add a unit test that parses
every assembled source with naga at test time:

* `crates/qbz/Cargo.toml`: `[dev-dependencies] naga = { version = "=28.0.0", features = ["wgsl-in"] }` (matching the `28.0.0` already in `crates/Cargo.lock` via wgpu, so no new tree).
* new `crates/qbz/src/shader_underlay/resources/tests.rs` (≤130 L):
  * for each `SHADER_SOURCES[i]` and for the assembled line_bed source
    (expose it as a `pub(in crate::shader_underlay) const LINEBED_SOURCE`
    in `mod.rs` so both the pipeline builder and the test use the same
    string — this is worth doing regardless), assert
    `naga::front::wgsl::parse_str(src).is_ok()`, printing the naga error
    on failure;
  * additionally run `naga::valid::Validator` over the parsed module so
    type errors, not just syntax errors, are caught;
  * assert each source contains exactly one `struct Uniforms` (guards
    against a scene keeping its local copy after the migration and
    silently producing a duplicate-declaration failure);
  * assert `vs_main` and `fs_main` are present in every scene source.

This is the first automated check of the shader sources at all — today
`const _: () = assert!(size_of::<Uniforms>() == 144)` guards only the Rust
half of the layout contract, as `shader_underlay/mod.rs:89-91` itself
admits ("WGSL is compiled at runtime (naga), so cargo cannot catch a
Rust/WGSL layout mismatch").

Manual verification after the change: run the app, open the immersive
view, select mode 1 (Plasma) and mode 5 (Line bed), and confirm no
`[shader]` errors in the log and that both scenes render — a naga parse
test does not prove the *visual* result is unchanged. Best evidence of
unchanged pixels: the moved text is byte-identical to what was removed, so
diff the assembled source against the pre-split file once by hand (write
both to files and `diff` ignoring whitespace-only chunk boundaries).

---

## 5. Execution order

1. Create `common/uniforms.wgsl`, `common/fullscreen_vs.wgsl`,
   `common/noise2d.wgsl`, `line_bed_terrain.wgsl` by **moving** text
   verbatim — no retyping, no reformatting.
2. Delete the moved ranges from `plasma.wgsl` and `line_bed.wgsl`.
3. Update `resources/mod.rs` and `resources/pipelines.rs`.
4. Add the naga test + `README.md`.
5. `cargo test -p qbz`, then run the app and switch to modes 1 and 5.
6. Only then consider migrating ambient/aurora/liquid_spectrum/
   spectral_ribbon/tunnel onto the same chunks (separate change).

---

## Risks

* **Shader-compile failures surface at runtime, not build time.** This is
  the dominant risk. `include_str!` and `concat!` are checked by cargo
  only as "the file exists"; the WGSL is parsed by naga inside
  `device.create_shader_module` (`pipelines.rs:18` and `:57`), which runs
  lazily — the pipelines are built on first use of a scene
  (`resources/mod.rs:60-64`: "lazy"), i.e. when a user selects that mode
  in the immersive view. A wrong concatenation order, a missing chunk, a
  duplicated `struct Uniforms`, or a chunk boundary that splits a token
  compiles clean and fails as a black underlay plus a wgpu validation
  error in the log, possibly on one user's machine only. The naga
  parse+validate test in §4 is the mitigation and should be treated as
  mandatory, not optional.
* **No error handling at the compile site.** Neither pipeline builder
  pushes a wgpu error scope; a shader-module validation error will surface
  through wgpu's default uncaptured-error handler. Splitting does not
  create this, but it raises the cost of a mistake. Consider wrapping both
  `create_shader_module` calls in
  `device.push_error_scope(wgpu::ErrorFilter::Validation)` / `pop` and
  logging, so a failure is a logged message rather than a hard failure —
  optional, separate change.
* **Chunk boundaries must end with a newline.** `concat!` does no
  joining. A chunk file whose last line lacks a trailing `\n` will glue
  its last token to the next chunk's first token, producing a confusing
  parse error far from the real cause. Every chunk file must end with a
  blank line; the naga test catches it, but state the requirement in the
  shaders README.
* **Naga error line numbers refer to the concatenated string, not the
  file.** After the split, "error at line 97" no longer maps to any file.
  Mitigate by keeping the chunk order fixed and documented, and by having
  the test, on failure, dump the assembled source to a temp file and print
  its path.
* **Silent divergence in the other direction.** Today each scene owns its
  `Uniforms`; a scene needing a different binding set cannot simply edit
  its own copy any more. That is the intended constraint, but it must be
  written down in the README or someone will "fix" a scene by re-inlining
  the struct and hit a duplicate-declaration error.
* **`vnoise` is not byte-identical between plasma and ambient** (local
  `uf` vs `w`). Semantically identical, but whoever migrates ambient onto
  `common/noise2d.wgsl` must confirm that rather than assume it. `blob` is
  genuinely different between the two files and must **not** be shared.
* **`SUBDIV` / `LINEBED_SUBDIV` coupling moves.** `line_bed.wgsl:52-55`
  documents that `SUBDIV` must match `LINEBED_SUBDIV` in Rust
  (`shader_underlay/mod.rs:60`). Moving the constant into
  `line_bed_terrain.wgsl` moves that hazard to a less obvious place;
  the comment must move with it intact, and `mod.rs:60` should name the
  new file.
* **line_bed's fallback margin is thin.** If the `line_bed_terrain.wgsl`
  chunk is dropped from the plan, line_bed lands at 127 L — three lines of
  headroom, and the next comment added to it puts the file back over
  budget.
