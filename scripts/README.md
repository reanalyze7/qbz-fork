# scripts/

Developer and acceptance scripts. Each is standalone; nothing here is
packaged or installed.

| Script | Purpose | Automated caller |
|---|---|---|
| `slint-run.sh` | Build the desktop app and exec the binary directly. | `.github/workflows/build-slint-selfhosted.yml` |
| `slint-live.sh` | Live-preview dev loop: edit `.slint` with no Rust rebuild. | none — run by hand |
| `qbzd-acceptance.sh` | End-to-end acceptance run against the `qbzd` daemon. | none — run by hand |
| `measure-cpu-window.sh` | Sample a running window's CPU over an interval. | none — run by hand |

`slint-run.sh` has a CI contract: its path and its env vars are referenced
from the workflow above and from the root `README.md`. Do not rename it or
change the meaning of `FAST` / `NORUN` / `THREADS` / `CODEGEN_UNITS` / `OPT`
without updating both.

---

## slint-run.sh

QBZ Slint — build with cargo, then run the BINARY DIRECTLY.

Why this exists (vs slint-dev.sh which does `cargo run`): `cargo run` launches
the app as a cargo-managed run target — the process inherits CARGO_* env and a
cargo launch context, which KDE plasma-systemmonitor surfaces by labelling the
RUNNING APP as "cargo" instead of "qbz-slint" (the kernel comm is still
qbz-slint; it's only the monitor's display name). Running the prebuilt binary
directly (no cargo wrapper) makes process monitors show it as `qbz-slint`,
cleanly separate from the cargo/rustc BUILD processes.

### The memory wall

A single RELEASE rustc for qbz-slint can hit ~20-24 GB. This box has **15 GB**
(measured 2026-08-26; the 30 GB this paragraph used to claim was wrong, and it
made the tier thresholds unreachable — see below). No hibernation, and it
HARD-FREEZES (power-cycle, lost work) on OOM/swap-thrash. The old fixed
`-Z threads=16` + opt-level=3 build SIGTERM'd at codegen whenever the desktop
held ~8-11 GB. To "skip the wall" this script:
 (a) SCALES rustc frontend threads + codegen-units + opt-level to the RAM that
     is actually free, so the compile FITS instead of being OOM-killed, and
 (b) runs the build under the `cargo-capped` cgroup so even a runaway dies
     cleanly (build killed) instead of freezing the whole box.

Tiers (auto, from `MemAvailable` as a FRACTION of installed RAM); override any
knob via env:
 >= 80% free  → FAST : threads=16 cgu=16  opt=3  (uncapped) — realistically a
                       TTY with nothing else running.
 >= 45% free  → SAFE : threads=2  cgu=256 opt=3  (a normal desktop session).
 below        → MIN  : threads=1  cgu=256 opt=2  (slow but never freezes).

The thresholds were absolutes (26 GB / 14 GB) written for a 30 GB machine. On
this 15 GB one, FAST was unreachable by construction and SAFE required ~93% of
total RAM free, so every single run landed in MIN and the tiering did nothing
but print a warning. As fractions they select a tier again.
NOTE: the SAFE/MIN tiers change codegen-units/opt-level, so the produced binary
is functionally identical but not byte-identical to the FAST/distribution build,
and switching tiers (or any RUSTFLAGS/profile knob) forces a one-time rebuild.

### Visual progress

Prints a start banner (wall-clock + tier), a live ⏱ ticker every 15s while the
long codegen phase is otherwise silent (elapsed / ETA / percent), and a final
banner with total build time. The ETA is learned: each successful build records
its duration per tier under ${XDG_CACHE_HOME:-~/.cache}/qbz-slint/, and the next
run of the same tier uses it as the estimate (no estimate on the first ever run
of a tier, or right after `cargo clean`). NO_TICKER=1 silences the live ticker.

### Usage

```sh
./scripts/slint-run.sh [extra app args]
 FAST=1                        ./scripts/slint-run.sh   # force the fast build
 THREADS=4 CODEGEN_UNITS=128 OPT=3 ./scripts/slint-run.sh   # manual override
 CAPPED=0                      ./scripts/slint-run.sh   # disable the cgroup cap
 NORUN=1                       ./scripts/slint-run.sh   # build only, don't exec
 NO_TICKER=1                   ./scripts/slint-run.sh   # no live progress ticker
```

---

## qbzd-acceptance.sh

qbzd P0 acceptance -- scripted checks (05-implementation-plan.md T16).

The human-only steps (J1 fresh-Pi journey, J2 desktop-handoff, the >24h JWT
soak, and the sign-off sheet) live in
qbz-nix-docs/qbz-daemon/acceptance/P0-acceptance.md. This script is the
automatable slice only.

SAFETY: runs the PREBUILT `qbzd` binary -- it never invokes cargo/rustc, so
it is safe to run on a box where a build is already in flight elsewhere
(global constraint: one build at a time, box-wide). It boots the daemon
against an ISOLATED scratch profile root (env-driven XDG_CONFIG_HOME /
XDG_DATA_HOME / XDG_CACHE_HOME) on a non-default port -- it never reads or
writes the real ~/.config/qbzd, ~/.local/share/qbzd or any desktop qbz
profile, and never touches systemd user units. Safe to run next to a real
qbzd/qbz install.

Usage:
  ./scripts/qbzd-acceptance.sh
  QBZD_BIN=/path/to/qbzd ./scripts/qbzd-acceptance.sh
  QBZD_TEST_PORT=28182 ./scripts/qbzd-acceptance.sh
set -euo pipefail

The run order lives in `scripts/qbzd-acceptance.sh`; the checks are in
`scripts/lib/qbzd-acceptance/`. The order is not free — `checks-lifecycle.sh`
stops the daemon and restarts it, so it must run after the API checks.

---

## measure-cpu-window.sh

measure-cpu-window.sh — sample CPU% of QBZ + WebKit children over a window

Usage:
  ./scripts/measure-cpu-window.sh [label] [duration_seconds]

Examples:
  ./scripts/measure-cpu-window.sh blur-on        # 30s sample, labelled "blur-on"
  ./scripts/measure-cpu-window.sh blur-off 60    # 60s sample, labelled "blur-off"
  ./scripts/measure-cpu-window.sh                # auto-label with timestamp

Reads /proc/PID/stat directly (no pidstat dependency). Samples per second.
CPU% is normalized per-core (100% = one fully busy core), matching the
convention used in `top` and in the bug report at issue #414/#415.

Output:
  - Live one-line update per second (qbz + webkit + total)
  - At end: mean, p50, p95, max for the whole window
  - CSV row appended to /tmp/qbz-cpu-measurements.csv for cross-run comparison

To compare runs:
  1. Pick a stable window state (e.g. 4K full-screen, home view, track playing)
  2. Run script, do NOT touch the window during the sample
  3. Change the variable (e.g. resize, toggle backdrop-filter), repeat
  4. cat /tmp/qbz-cpu-measurements.csv

---

## slint-live.sh

The fast path for UI work, and the only change here that alters the *shape* of
the problem rather than shaving a percentage off it.

`slint-build` normally generates a ~1.6M-line Rust module from `ui/app.slint`.
That one crate, `qbz_ui`, is where this project's build time and its ~30 GB
rustc peak live — and every `.slint` edit invalidates it, which is why cargo
caching never helps the case we actually hit.

With `SLINT_LIVE_PREVIEW=1` in the build environment **and** the `live-preview`
Cargo feature, `slint-build` emits stubs that load the `.slint` tree at runtime
instead. The expensive module is never produced. A `.slint` edit then needs no
Rust rebuild at all — unless you change the Rust ↔ UI bridge (globals,
callbacks, struct fields), which the Rust side references by name.

Both halves are required. The env var alone gives you stubs with no interpreter
to run them; the feature alone changes nothing.

**Not shippable.** The binary reads `ui/*.slint` from the source tree at
startup, so it only runs from this checkout. Use `slint-run.sh` for anything
you intend to keep.

```sh
./scripts/slint-live.sh              # dev profile
FASTCC=1 ./scripts/slint-live.sh     # + Cranelift backend
RELEASE=1 ./scripts/slint-live.sh    # release profile
```

`FASTCC=1` swaps LLVM for Cranelift on the dev profile. It compiles much
faster and generates much worse code — so it is opt-in, never the default:
this app decodes and resamples audio on a realtime thread. Note also that
Cranelift is a *backend*, while the `qbz_ui` peak is rustc *frontend* work, so
it speeds up the other ~30 crates and not the expensive one. Live preview is
what fixes that crate.

It is selected through `CARGO_PROFILE_DEV_CODEGEN_BACKEND` in the script, not
through a profile in `crates/Cargo.toml`, and that is not a style choice.
`codegen-backend` is an unstable profile key, and cargo rejects the **whole
manifest** the moment it parses one it does not understand — including for
jobs that would never use that profile. A checked-in `[profile.dev-fast]`
therefore broke the `build-qbzd` CI job, which builds this same workspace on
**stable** to hold its glibc 2.35 floor. Environment variables only exist for
the duration of this script, so no other build ever sees them.

It needs the component once:

```sh
rustup component add rustc-codegen-cranelift-preview --toolchain nightly-2026-06-23
```
