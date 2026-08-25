# AUDIT — 130-line budget: inventory, exceptions, index

Scope: the whole repo at branch `refactor/split-files-130-lines`.
Method: `git ls-files -z | xargs -0 grep -Il '' | wc -l`, then filtering.
No source file was modified in producing this document.

---

## Part 1 — Re-derived inventory

### 1.1 Method and raw result

`git ls-files` tracks 4602 paths. Piping all of them through `wc -l` produces
garbage for binary content (a `.ttf` is reported at 59215 "lines" because `wc`
counts `0x0A` bytes in the glyph tables). The list was therefore first passed
through `grep -Il ''`, which keeps only files GNU grep considers text, and then
`crates/vendor/**` was dropped. That leaves **69 tracked text files over 130
lines**.

A boundary check was run for the classic `wc -l` off-by-one (a file whose last
line has no terminating newline is reported one short): no tracked text file in
the 128–130 range lacks a trailing newline, so nothing sits just under the line
by accident.

### 1.2 Exclusions, with reasons

**`crates/vendor/**` — excluded.** Vendored upstream Slint 1.16.1 and
femtovg 0.23.2 sources (`i-slint-core`, `i-slint-renderer-skia`,
`i-slint-renderer-femtovg`, `i-slint-common`, `femtovg`). 100+ files over
budget, the largest being `i-slint-core-1.16.1/item_tree.rs` at 2863 lines.
The rule is a rule about *code this project writes*. Reformatting a vendored
tree destroys the ability to diff against upstream and to re-vendor a new
version, which is the entire point of vendoring. Out of scope permanently, not
"deferred".

**Binary files — excluded as measurement artifacts.** `.ttf` (fonts under
`static/fonts/**` and `crates/qbz-ui/ui/assets/fonts/**`), `.png`
(`packaging/flatpak/screenshots/**`, `flatpak/screenshots/**`), `.icns`
(`packaging/icons/icon.icns`). These have no lines; `wc` was counting bytes
that happen to be `0x0A`. Removed by the `grep -Il` filter above.

**Generated / machine-maintained files — excluded.** These *are* text and do
survive the filter, but nothing about them is hand-authored:

| Path | Lines | Why excluded |
|---|---|---|
| `crates/Cargo.lock` | 10484 | Cargo-generated; editing it is a bug |
| `crates/qbz-ui/translations/*/LC_MESSAGES/qbz-ui.po` (8 files) | 611–4845 | gettext catalogs; structure is one entry per msgid, dictated by the extractor |
| `static/db/dac_database_seed_500_en.json` | 11473 | data seed, one record per DAC |
| `docs/openapi.yaml` | 1199 | API surface description; the shape is the API's, not a design choice |

**Build-script output — checked, none found.** `crates/target/` and `target/`
are in `.gitignore` and confirmed absent from `git ls-files` (0 tracked paths
under either). The many files named `build.rs` inside `crates/qbz/src/**`
(e.g. `sidebar/entry_build.rs`, `settings/snapshot/build.rs`) are *not* Cargo
build scripts — they are ordinary modules that happen to be named `build`
because they build a view model. The only real Cargo build scripts are
`crates/qbz-ui/build.rs` and `crates/qbz/build.rs`, both under budget. The
Slint-generated Rust (`slint-build` output for `ui/app.slint`) lands in the
ignored target dir and is not committed. Nothing generated is tracked.

**Vector assets — excluded.** 16 `.svg` files over budget (`mexico-flag.svg`
1721, `qobuz-logo-filled.svg` 723, `Tux.svg` 438, down to
`copy-list-sheet.svg` 139). These are path data. Note in passing that several
exist twice — `static/mexico-flag.svg` and
`crates/qbz-ui/ui/assets/mexico-flag.svg` are byte-identical duplicates, as are
the `gandalf`, `hi-res` and `qobuz-logo-filled` pairs. That is a
deduplication question, not a line-count one, and is out of scope here.

### 1.3 What remains

After those exclusions, 30 files. They fall into four groups.

**(a) Code — 15 files. The rule plainly applies.**
Rust (3), Slint (6), WGSL (4), shell (3). Full list in the Part 3 index.

**(b) Cargo manifests — 2 files.** `crates/qbz/Cargo.toml` (203),
`crates/Cargo.toml` (162). A manifest is a declaration list, not a script;
there is no control flow to follow and no cohesion to be gained. Cargo also
offers no include mechanism for splitting one — the workspace `Cargo.toml`
already *is* the factoring device (`[workspace.dependencies]`), and
`crates/qbz/Cargo.toml`'s length is a direct function of the crate having many
dependencies. Splitting is not possible, and the rule's underlying goal
(a file whose whole responsibility fits in one screen) is not served by a
dependency table. **Out of scope.** If the length bothers anyone, the real
lever is fewer dependencies, which is a different project.

**(c) Documentation — 15 files.** `README.md` (507), `CONTRIBUTING.md` (134),
and 12 files under `docs/release-*/` (`CHANGELOG_FULL.md`,
`CHANGELOG_HIGHLIGHTS.md`, `CHANGELOG.md`, `TAG_DETAILS.md`; 131–834 lines),
plus `packaging/flatpak/com.blitzfc.qbz.metainfo.xml` (345).

Position on `docs/**/CHANGELOG*.md`: **do not split, and do not treat them as
in scope at all.** The reasoning is that the rule exists to bound the amount of
*interdependent logic* a reader has to hold in their head at once. A changelog
has none: it is an append-only list of independent entries, read by search or
by scanning, and every entry is comprehensible without any other entry. The
130-line ceiling would also be actively harmful here, because these files are
historical records of already-shipped releases. Splitting
`docs/release-1.1.8/CHANGELOG_FULL.md` into seven files would break every
existing link to it and rewrite the record of a release that shipped long ago,
in exchange for nothing. `README.md` is the one document where a length
argument has some force — a 507-line README is genuinely hard to navigate —
but the fix there is editorial (move sections to `docs/`), not mechanical
line-counting, and it belongs to whoever owns the docs.
`com.blitzfc.qbz.metainfo.xml` is an AppStream manifest whose bulk is
`<release>` entries mirroring the changelog; same argument, plus the schema
forbids splitting it.

**(d) CI and packaging YAML — 11 files.** 8 workflows (404, 330, 292, 215,
185, 185, 181, 147), `.github/ISSUE_TEMPLATE/bug_report.yml` (181), and
`snapcraft.yaml` / `packaging/snap/snapcraft.yaml` (161 each, duplicates of
each other).

### 1.4 Position on `.github/workflows/*.yml`

The question asked is whether a CI workflow is a "script" under this rule. It
is a genuinely close call and the honest answer has two halves.

**It is script-like.** `release-linux.yml` at 404 lines contains real
imperative logic — multi-line `run:` blocks with `bash` in them, conditional
steps, matrix expansion, artifact handoff between three jobs (`build` at
lines 37–214, `build-qbzd` at 215–246, `package` at 247–404). Someone
debugging a failed release has to read it top to bottom exactly the way they
would read a shell script. On that basis it is not obviously exempt.

**But splitting it costs more than it returns.** GitHub Actions offers exactly
two factoring mechanisms, and neither produces the outcome the rule wants:

- *Composite actions* (`.github/actions/*/action.yml`) can extract a run of
  steps, but they cannot carry a job's `runs-on`, `strategy`, `permissions`,
  `services`, or `outputs`, and they cannot contain jobs. So a composite can
  absorb the middle of a job but never a job. This repo already uses them —
  `qbz-linux-runner-prep` (83 lines) and `qbz-channel-publish` (99 lines) —
  which is evidence that where extraction *is* natural, it has already been
  done. Both are currently referenced only from `release-channel.yml`.
- *Reusable workflows* (`workflow_call`) can extract a whole job, but each
  reusable workflow runs on its own runner. Extracting `build` from
  `release-linux.yml` would mean the compiled binary now has to cross a
  workflow boundary via `upload-artifact` / `download-artifact`, and the
  shared secrets and `concurrency` group have to be re-plumbed as explicit
  `secrets:` and `inputs:`. That adds runner minutes, adds artifact
  round-trips, and adds a layer of parameter indirection to the one file
  people read when a release is broken.

**Verdict: out of scope, deliberately.** The cost is real minutes and real
indirection on the release path; the gain is a line count. A cheaper move that
captures most of the readability benefit without any of the cost: the eight
release workflows share large stretches of step text (all eight check out,
install a toolchain, and upload artifacts the same way), so extracting *those
shared runs* into two or three more composite actions would shrink several
files at once and remove genuine duplication. That is worth doing on its own
merits, not to satisfy a line budget. `bug_report.yml` is a form schema with
no logic at all — clearly out of scope. `snapcraft.yaml` is a package recipe
in the same category as a Cargo manifest; the notable thing about it is that
it exists twice at 161 lines each, which is a duplication problem, not a
length problem.

---

## Part 2 — The two documented exceptions

### 2.1 `crates/qbz/src/main.rs` — 998 lines

**Claim (module doc, lines 17–41), two independent reasons:**
1. The `mod x;` list must live textually in the crate root.
2. `fn main()`'s body is a sequential boot procedure holding the tokio
   `Runtime` and its `_enter` guard.

**Reason 1 is technically sound and fully covers what it claims.** Rust has no
way to declare `mod x;` from a non-root file such that `x` resolves as
`crate::x`; the only workarounds are `include!()` (splicing tokens, harder to
grep) or rewriting every `crate::x::y` path in the crate. 137 `mod`
declarations were counted. With the `#[cfg]` attributes, the blank grouping
lines, the `pub(crate) use ...::*;` re-export lines and the two explanatory
comment blocks, lines 44–224 — **181 lines** — are this section. Add the
module doc (1–41) and the imports/const at 226–240 and the pre-`fn main()`
part of the file is **240 lines that genuinely cannot move**. Reason 1 alone
already puts the file at ~1.8× budget with zero remedy available.

**Reason 2 is sound as a principle but no longer describes the file.** The
lifetime argument is correct where it applies: `tokio_rt` and `_enter` must be
created and dropped in this frame, and the same is true of `window`,
`app_runtime`, `image_cache` and `settings_ctx` as they are borrowed by the
trailing `wire_*` calls. But `Runtime::enter()` sets a *thread-local*. Any
function called from `fn main()`'s frame, at any depth, sees
`Handle::current()` for as long as the guard is alive in that frame. So the
guard justifies keeping the *bindings* in `main`; it does not justify keeping
the 700 lines that merely *use* them inline.

Structural read of `fn main()` (lines 242–998, **757 lines**):

- 242–421 (~180 lines): the actual boot procedure — UI-scale env var (which
  must precede any thread spawn), logger install, `deep_link::capture_argv`,
  `Runtime::new` + `enter`, single-instance acquire, backend selection,
  `AppWindow::new`, the renderer-sentinel and DPR-probe timers. Order-critical
  and handle-creating. **Irreducible.**
- 422–930 (~509 lines): a long run of `{ ... }` blocks that seed Slint globals
  from `ui_prefs` — app-background availability (422), shell/sidebar restore
  (508), appearance toggles (546), title-bar chrome (566), notifications
  (585), theme/palette (607–657), visualizer seed (677), the wgpu rendering
  notifier (694–729), the MusicBrainz enable spawn (736), and the offline-mode
  reactor at 771–931 (161 lines). Every one of these reads `window` and
  `boot_prefs` (plus `use_gpu_renderer`, or the standard five handles) and
  creates nothing that outlives it. **Movable.** They are the same shape as
  the `wire_*` functions already extracted at 984–995 and would take the same
  parameter list.
- 932–964 (33 lines): core init + saved-session restore. Same shape.
  **Movable.**
- 966–998: the boundary comment and the 12 `wire_*` calls. Correct as-is.

The file's own boundary comment at 966 asserts that everything above it is the
handle-creating boot procedure. That was true when written; it has drifted.
The pref-seeding blocks accumulated *above* the boundary because each new
setting was added next to the last one, not because any of them belongs there.

**Verdict: exception valid, but the file has drifted; roughly 540 lines could
still move out** (422–964, less a handful of binding lines such as `let
app_runtime` at 663, `let image_cache` at 751 and `let settings_ctx` at 754
which must stay). Extracting them as, say, `seed_appearance_from_prefs`,
`seed_shell_from_prefs`, `install_rendering_notifier`, `wire_offline_reactor`
and `wire_startup_restore` — each further split to ≤130 lines, following the
existing `wire_*_partN` convention — would leave `main.rs` at roughly
**430–460 lines**: 240 irreducible mod-list/doc/imports plus ~180 lines of
true boot sequence plus the call tail. Still 3.5× budget, still exempt under
reason 1, but honestly so. **Reason 2 should be narrowed in the doc comment to
describe lines 242–421 only, rather than everything before the wiring calls.**

Caveat, stated because it changes the risk assessment rather than the
conclusion: the refactor plan referenced in the doc records that `cargo check`
is not available for this work. Moving 540 lines of closure-capturing Slint
callbacks without a compiler is exactly the kind of change that silently
breaks startup. The drift is real and worth recording; acting on it should
wait for a build.

### 2.2 `crates/qbz/src/artwork/target.rs` — 298 lines

**Claim: "a pure enum" — no I/O, no logic beyond `decode_size`'s match.**

Verified against the current file:

- 1–11: module doc stating the exception.
- 12: one `use` (`scaled_decode`, `DECODE_SIZE`).
- 15–262: `#[derive(Clone)] pub enum ArtworkTarget` — **248 lines**, made up
  of **90 variants** and **149 lines of `///` doc comments** on them.
- 264–298: `impl ArtworkTarget` — **35 lines**, containing exactly one
  function, `decode_size`, which is a single `match` returning 48/96/160 or
  `DECODE_SIZE`.

The claim is accurate. There is no I/O, no second impl, no trait impl, no
tests, no free functions.

Quantifying the irreducible part: an enum is one Rust item and its variants
cannot be distributed across files. 248 lines therefore cannot be split by any
mechanism short of redesigning the type. The only extraction physically
available is moving `impl ArtworkTarget` into a sibling (`target/mod.rs` +
`target/decode_size.rs`), which removes 35 lines and leaves 263 — **still more
than double the budget**. So the split is not merely undesirable, it is
ineffective: it cannot reach the target and it separates the one function that
matches on the enum from the enum it matches on.

Note that half the file is documentation. 149 of 298 lines are `///` comments
explaining which Slint model index each variant addresses. Stripping them
would bring the file to ~150 lines, but deleting good documentation to satisfy
a line count would be a straightforwardly bad trade and is not recommended.

**Verdict: exception valid as stated.** No lines should move. If anything, the
doc comment could be strengthened with the point that the only available
extraction (the `impl`) would not reach 130 anyway, which is the argument that
actually closes the question.

---

## Part 3 — Index of files over budget

Line counts as of this audit. "Plan" names the file in `split-plans/`.

### Code — in scope

| Path | Lines | Category | Plan | Disposition |
|---|---|---|---|---|
| `crates/qbz/src/main.rs` | 998 | Rust | — | **exception** (valid; ~540 lines have drifted in and could move — see 2.1) |
| `crates/qbz/src/artwork/target.rs` | 298 | Rust | — | **exception** (valid; do not split) |
| `crates/qbz-ui/ui/shell/sidebar/SidebarRow.slint` | 242 | Slint | `SidebarRow.slint.md` | split |
| `crates/qbz-ui/ui/app.slint` | 237 | Slint | `app.slint.md` | split |
| `scripts/qbzd-acceptance.sh` | 237 | shell | `scripts.md` | split |
| `scripts/measure-cpu-window.sh` | 215 | shell | `scripts.md` | split |
| `crates/qbz-ui/ui/state/appearance_state.slint` | 197 | Slint | `appearance_state.slint.md` | split |
| `crates/qbz-ui/ui/shaders/tunnel.wgsl` | 185 | WGSL | `shaders-tunnel-ambient.md` | split |
| `crates/qbz-ui/ui/shaders/ambient.wgsl` | 179 | WGSL | `shaders-tunnel-ambient.md` | split |
| `crates/qbz-ui/ui/shell/HeaderSearch.slint` | 164 | Slint | `HeaderSearch.slint.md` | split |
| `crates/qbz-ui/ui/shaders/plasma.wgsl` | 153 | WGSL | `shaders-plasma-linebed.md` | split |
| `crates/qbz-ui/ui/state/local_library_state.slint` | 152 | Slint | `local_library_state.slint.md` | split |
| `crates/qbz-ui/ui/shell/HeaderBar.slint` | 147 | Slint | `HeaderBar.slint.md` | split |
| `crates/qbz-ui/ui/shaders/line_bed.wgsl` | 144 | WGSL | `shaders-plasma-linebed.md` | split |
| `scripts/slint-run.sh` | 141 | shell | `scripts.md` | split |
| `crates/qbz/src/playback/meta/artwork.rs` | 131 | Rust | `playback-one-liners.md` | split (1 line over) |
| `crates/qbz/src/playback/local/folder.rs` | 131 | Rust | `playback-one-liners.md` | split (1 line over) |

Every in-scope code file is claimed by exactly one of the nine plans, or is
one of the two exceptions. **Coverage is complete — there is no orphaned
code file over budget.**

The two 131-line Rust files are one line over. Both end in a nested closure
inside a `spawn`, so the last few lines are closing braces; a plan that
extracts one small helper clears them. Worth stating plainly: these are not
readability problems, they are arithmetic. Splitting them buys nothing except
compliance, and a plan that turns one 131-line coherent file into two
incoherent 70-line files would make the codebase worse. The right move is the
smallest one that works.

### Manifests, docs, packaging — out of scope

| Path(s) | Lines | Category | Disposition |
|---|---|---|---|
| `crates/qbz/Cargo.toml` | 203 | manifest | out of scope — no split mechanism; length tracks dependency count |
| `crates/Cargo.toml` | 162 | manifest | out of scope — this file *is* the factoring device |
| `README.md` | 507 | docs | out of scope for this rule; editorial trim is a separate call |
| `CONTRIBUTING.md` | 134 | docs | out of scope |
| `docs/release-1.1.8/CHANGELOG_FULL.md` | 834 | docs | out of scope — append-only historical record |
| `docs/release-1.1.10/CHANGELOG_FULL.md` | 755 | docs | out of scope |
| `docs/release-2.0.0/CHANGELOG.md` | 364 | docs | out of scope |
| `docs/release-1.1.8/CHANGELOG_HIGHLIGHTS.md` | 348 | docs | out of scope |
| `docs/release-2.0.2/CHANGELOG.md` | 259 | docs | out of scope |
| `docs/release-1.1.19/CHANGELOG_FULL.md` | 259 | docs | out of scope |
| `docs/release-1.2.13/CHANGELOG_FULL.md` | 256 | docs | out of scope |
| `docs/release-1.2.5/CHANGELOG_FULL.md` | 216 | docs | out of scope |
| `docs/release-1.1.15/CHANGELOG_FULL.md` | 190 | docs | out of scope |
| `docs/release-1.2.7/TAG_DETAILS.md` | 140 | docs | out of scope |
| `docs/release-1.2.6/TAG_DETAILS.md` | 134 | docs | out of scope |
| `docs/release-1.1.19/TAG_DETAILS.md` | 131 | docs | out of scope |
| `packaging/flatpak/com.blitzfc.qbz.metainfo.xml` | 345 | AppStream XML | out of scope — schema forbids splitting; bulk is release history |
| `.github/workflows/release-linux.yml` | 404 | CI | out of scope — see 1.4; extract shared steps as composites instead |
| `.github/workflows/release-linux-aarch64.yml` | 330 | CI | out of scope — same |
| `.github/workflows/release-snap.yml` | 292 | CI | out of scope |
| `.github/workflows/release-macos.yml` | 215 | CI | out of scope |
| `.github/workflows/release-gentoo.yml` | 185 | CI | out of scope |
| `.github/workflows/release-aur.yml` | 185 | CI | out of scope |
| `.github/workflows/release-updater-manifest.yml` | 181 | CI | out of scope |
| `.github/workflows/release-flatpak.yml` | 147 | CI | out of scope |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | 181 | form schema | out of scope — no logic |
| `snapcraft.yaml` | 161 | package recipe | out of scope (duplicate of the next; dedupe, don't split) |
| `packaging/snap/snapcraft.yaml` | 161 | package recipe | out of scope |

### Excluded — not counted

| Group | Files | Reason |
|---|---|---|
| `crates/vendor/**` | 100+ | third-party Slint/femtovg source |
| `.ttf` / `.png` / `.icns` | ~90 | binary; `wc -l` counts `0x0A` bytes |
| `.svg` | 16 | vector path data |
| `.po` | 8 | gettext catalogs, extractor-shaped |
| `crates/Cargo.lock` | 1 | Cargo-generated |
| `static/db/dac_database_seed_500_en.json` | 1 | data seed |
| `docs/openapi.yaml` | 1 | API description |
| build-script output | 0 | none tracked; `target/` and `crates/target/` are ignored and absent from `git ls-files` |
