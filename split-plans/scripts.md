# scripts/: `qbzd-acceptance.sh`, `measure-cpu-window.sh`, `slint-run.sh`

Three shell scripts over the 130-line budget:

| file | lines | verdict |
|---|---|---|
| `scripts/qbzd-acceptance.sh` | 237 | real split — helper lib + per-phase check files |
| `scripts/measure-cpu-window.sh` | 215 | real split — pure / I-O / render, exactly the CLAUDE.md seam |
| `scripts/slint-run.sh` | 141 | **partial exception** — 43 of its 141 lines are the doc header; move the prose, leave the pipeline whole |

---

## Facts established first (apply to all three)

**Invocation contracts** (grepped `.github/`, `README.md`, `docs/`, `packaging/`, `scripts/`):

- `scripts/slint-run.sh` **is a CI contract**. `.github/workflows/build-slint-selfhosted.yml:47`
  runs `NORUN=1 CAPPED=1 THREADS=2 CODEGEN_UNITS=256 OPT=3 ./scripts/slint-run.sh`
  from the repo root. `README.md:407–423` documents the same path plus
  `FAST=1`, `NORUN=1`, `THREADS/CODEGEN_UNITS/OPT`. Path, env-var names, and
  the `NORUN=1 → exit 0` behaviour are frozen.
- `scripts/qbzd-acceptance.sh` has **no** CI or README caller. It is run by hand
  (`./scripts/qbzd-acceptance.sh`, optionally `QBZD_BIN=…`, `QBZD_TEST_PORT=…`).
  Its exit contract is still real: `0` = all checks passed, `1` = a `fail`,
  and it is referenced by `qbz-nix-docs/qbz-daemon/acceptance/P0-acceptance.md`
  (per its own header) as the automatable slice of a sign-off procedure.
- `scripts/measure-cpu-window.sh` has no caller anywhere. Only a changelog line
  (`docs/release-1.2.13/CHANGELOG_FULL.md:241`) mentions it. Positional args
  `[label] [duration_seconds]` and `exit 2` on a bad duration / `exit 1` on
  "no qbz process" are the only contract, plus the CSV schema written to
  `/tmp/qbz-cpu-measurements.csv`.

**Packaging**: nothing packages `scripts/`. The flatpak manifest mentions
`scripts/update-flathub.sh` only in a comment; no `.gitattributes` exists, so
`git archive` and the CI checkout both carry any new file automatically.
**No packaging list needs updating for any of the three splits.**

**No existing convention to follow**: `scripts/` is flat (12 `.sh` files, no
subdirectory), and no script in the repo currently `source`s anything —
`grep -rn '^\s*\(source\|\.\) ' scripts/` is empty. So this plan *establishes*
the convention: `scripts/lib/<script-name>/<part>.sh`, one subdirectory per
entry script, no shared cross-script library until two scripts actually need
the same function.

**No shellcheck anywhere** in the repo or CI, so `# shellcheck source=` hints
are optional; they are listed below anyway because they cost one comment line
and make a future lint adoption free.

**The sourcing idiom** — used verbatim in all three entry scripts, placed
*after* `set -euo pipefail` and *before* any `cd`:

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
```

`${BASH_SOURCE[0]}` (not `$0`) so the value is right whether the script is
executed, sourced, or reached through a relative path; the `cd … && pwd`
absolutises it so a later `cd` in the script cannot invalidate it.

**`set -euo pipefail` across sourced files**: `source` runs in the *same* shell,
so the entry script's `set -euo pipefail` is already in force while each lib
file is read and while its functions later run. Therefore:

- the lib files **must not** repeat `set -euo pipefail` (harmless today, but it
  would silently re-enable options a caller had deliberately turned off), and
- the lib files **must not** contain top-level commands that can fail — under
  `errexit` a failing top-level line in a sourced file kills the *entry script*
  at source time, before any check has run. Every moved block therefore becomes
  a **function**; only function definitions and `readonly`-style constants sit
  at the top level of a lib.
- `set +e` / `set -e` toggling inside a moved function keeps working exactly as
  it does today (it is shell-global, not function-scoped) — `qbzd-acceptance.sh`
  depends on this in five places.
- One `errexit` trap to respect: inside a function invoked as a *condition*
  (`if f; then`, `f || x`, `f && x`), `errexit` is suspended for the whole
  function body. None of the extractions below introduce such a call site;
  every phase function is called as a plain statement. This is a rule to keep,
  not just an observation.

**Permissions**: entry scripts stay `775` and keep their shebang. Lib files are
`644` and **not** executable — they are never run, only sourced; an executable
bit on them invites someone to run one directly, where it does nothing (or, in
`qbzd-acceptance`'s case, defines functions into a shell that then exits). Give
each lib a `# shellcheck shell=bash` first line instead of a shebang, so editors
and any future linter still pick the dialect.

---

## 1. `scripts/qbzd-acceptance.sh` (237 → ~55 + 5 libs)

### Why is this file long?

Genuinely multi-responsibility. It is four different things stacked:
preflight, an isolated-environment builder, a daemon lifecycle manager, and
then thirteen independent assertions. The assertions are the bulk (lines
131–237, 107 lines) and they are *not* one procedure — each is a self-contained
"do X, assert Y, `fail` with a spec reference" block that happens to share a
running daemon. That is a list, and a list splits.

### Seams

| lines | block | becomes |
|---|---|---|
| 1–22 | header + `set -euo pipefail` | trimmed, stays in entry |
| 24–26 | `ROOT` / `QBZD_BIN` / `PORT` | stays in entry |
| 28–33 | `fail()` + `curl`/`python3`/`timeout`/binary preflight | `lib/common.sh` |
| 35–56 | scratch XDG root, `$HOME` guard, `QBZD_HOST`/`LOGFILE`/`DAEMON_PID` | `lib/scratch-env.sh` |
| 58–89 | `kill_and_confirm`, `cleanup`, port-is-free guard | `lib/daemon.sh` |
| 91–129 | `qbzd()`, `write_config()` (here-doc), `start_daemon`, `stop_daemon` | `lib/daemon.sh` |
| 131–145 | boot, exit-code table, unknown-key warning | `lib/checks-boot.sh` |
| 147–207 | status / ping / info / config-show / settings roundtrip / route budget | `lib/checks-api.sh` |
| 209–234 | daemon-down codes, instance lock, non-tty setup | `lib/checks-lifecycle.sh` |
| 236–237 | final `stop_daemon` + success line | stays in entry |

### New files

```
scripts/qbzd-acceptance.sh                      ~55   (775, entry)
scripts/lib/qbzd-acceptance/common.sh           ~16   (644)
scripts/lib/qbzd-acceptance/scratch-env.sh      ~30   (644)
scripts/lib/qbzd-acceptance/daemon.sh           ~72   (644)
scripts/lib/qbzd-acceptance/checks-boot.sh      ~24   (644)
scripts/lib/qbzd-acceptance/checks-api.sh       ~70   (644)
scripts/lib/qbzd-acceptance/checks-lifecycle.sh ~34   (644)
scripts/lib/qbzd-acceptance/README.md
```

Every moved block is wrapped in a function named after its `echo "== … =="`
banner; the banner `echo` moves *into* the function so the entry reads as a
table of contents:

- `common.sh`: `fail()`, `require_tools()` (lines 30–33 verbatim, `$QBZD_BIN` read as a global).
- `scratch-env.sh`: `setup_scratch_env()` — the `mktemp -d`, the three
  `export`s, `mkdir -p`, the `$HOME/.config` refusal (50–52), and the
  `QBZD_HOST`/`LOGFILE`/`DAEMON_PID` initialisation.
- `daemon.sh`: `kill_and_confirm()`, `cleanup()`, `assert_port_free()`
  (86–89), `qbzd()`, `write_config()`, `start_daemon()`, `stop_daemon()`.
- `checks-boot.sh`: `check_boot()`, `check_exit_code_table()`, `check_unknown_key_warning()`.
- `checks-api.sh`: `check_status_shape()`, `check_ping_info_shape()`,
  `check_config_show_port()`, `check_settings_roundtrip()`, `check_route_budget()`.
- `checks-lifecycle.sh`: `check_daemon_down_codes()`, `check_instance_lock()`,
  `check_setup_non_tty()`.

### Scope: global vs `local`

Must stay **global** (assigned in a lib, read by another lib or by the `EXIT` trap):
`ROOT`, `QBZD_BIN`, `PORT`, `SCRATCH`, `XDG_*`, `QBZD_HOST`, `LOGFILE`, `DAEMON_PID`.
`start_daemon` assigning `DAEMON_PID=$!` and `stop_daemon` clearing it are the
whole reason `cleanup` works — neither may become `local`.

Can and should become `local`: `rc_usage` (139), `status_json`/`rc_status_needs_auth`
(154–155), `cfg_port` (182), `BUNDLE`/`out` (195–197), `code`/`ocode` (204–206),
`rc_ping`/`rc_status` (213–215), `rc_second` (224), `rc_setup` (232). `pid` in
`kill_and_confirm` is already `local`.

Note the here-doc in `write_config` (98–105) interpolates `$PORT` — it stays an
unquoted `<<EOF` and `$PORT` stays global. Do not turn it into a data file: it
is five lines with one substitution, and a separate `.toml` template would need
`envsubst` or `sed` to be useful.

### Entry script after the split

```bash
#!/usr/bin/env bash
# qbzd P0 acceptance -- scripted checks (05-implementation-plan.md T16).
# … 12-line trimmed header: SAFETY paragraph + Usage block kept verbatim,
#   the "human-only steps" paragraph kept, the rest → lib README …
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
QBZD_BIN="${QBZD_BIN:-$ROOT/crates/target/release/qbzd}"
PORT="${QBZD_TEST_PORT:-28182}"

# shellcheck source=scripts/lib/qbzd-acceptance/common.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/common.sh"
# shellcheck source=scripts/lib/qbzd-acceptance/scratch-env.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/scratch-env.sh"
# shellcheck source=scripts/lib/qbzd-acceptance/daemon.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/daemon.sh"
# shellcheck source=scripts/lib/qbzd-acceptance/checks-boot.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/checks-boot.sh"
# shellcheck source=scripts/lib/qbzd-acceptance/checks-api.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/checks-api.sh"
# shellcheck source=scripts/lib/qbzd-acceptance/checks-lifecycle.sh
source "$SCRIPT_DIR/lib/qbzd-acceptance/checks-lifecycle.sh"

require_tools
setup_scratch_env
trap cleanup EXIT          # after scratch-env, so $SCRATCH exists
assert_port_free

check_boot
check_exit_code_table
check_unknown_key_warning
check_status_shape
check_ping_info_shape
check_config_show_port
check_settings_roundtrip
check_route_budget
check_daemon_down_codes
check_instance_lock
check_setup_non_tty

stop_daemon
echo "ALL SCRIPTED CHECKS PASSED"
```

Ordering constraint worth stating: `trap cleanup EXIT` must be installed
*after* `setup_scratch_env` (today it is line 84, after line 43's `mktemp`) —
if the trap fires before `$SCRATCH` is set, `rm -rf "$SCRATCH"` runs on an
empty string. `set -u` makes that an error rather than an `rm -rf /`, but the
ordering is load-bearing either way and the entry script must keep it.

The other ordering constraint: `check_daemon_down_codes` stops the daemon and
`check_instance_lock` starts it again. The phases are not freely reorderable;
the lib README must say so.

---

## 2. `scripts/measure-cpu-window.sh` (215 → ~48 + 5 libs)

### Why is this file long?

Multi-responsibility, and it lands on the pure ↔ I/O ↔ render seam almost
without effort: process discovery and `/proc` reads are I/O, `summarize` is
pure arithmetic over a list of numbers, and roughly 45 lines do nothing but
`printf`. The 25-line doc header is the fourth chunk.

### Seams

| lines | block | becomes |
|---|---|---|
| 1–26 | doc header | trimmed to ~10 (usage + CSV pointer); the "To compare runs" recipe → `scripts/README.md` |
| 27–37 | `set`, `LABEL`/`DURATION`/`LOG_CSV`/`CLK_TCK`, duration validation | stays in entry |
| 39–77 | `find_qbz_pid`, `find_webkit_pids`, the no-process `exit 1`, `mapfile` | `lib/discover.sh` |
| 79–101 | `read_jiffies`, `read_window_size` | `lib/procstat.sh` |
| 103–114 | start banner | `lib/report.sh` |
| 116–130 | initial snapshot, `samples_*` arrays, column headers | `lib/sample.sh` |
| 132–164 | the sampling loop | `lib/sample.sh` |
| 166–180 | `summarize` | `lib/stats.sh` (pure) |
| 182–203 | summary rendering | `lib/report.sh` |
| 205–211 | CSV header + append | `lib/report.sh` |
| 213–215 | footer | `lib/report.sh` |

### New files

```
scripts/measure-cpu-window.sh              ~48   (775, entry)
scripts/lib/measure-cpu/discover.sh        ~44   (644)
scripts/lib/measure-cpu/procstat.sh        ~26   (644)
scripts/lib/measure-cpu/sample.sh          ~54   (644)
scripts/lib/measure-cpu/stats.sh           ~20   (644)
scripts/lib/measure-cpu/report.sh          ~52   (644)
scripts/lib/measure-cpu/README.md
```

- `discover.sh`: `find_qbz_pid()` (42–61 verbatim), `find_webkit_pids()`
  (72–74), plus `resolve_targets()` wrapping 63–67 and 76–77 — it sets the
  globals `QBZ_PID` and `WEBKIT_PIDS` and keeps the `exit 1` on "no qbz
  process found", which is part of the contract.
- `procstat.sh`: `read_jiffies()` and `read_window_size()` unchanged. Both are
  read-only `/proc`/log accessors with no globals — this is the file that will
  be reused if a second sampler ever appears.
- `sample.sh`: `sample_window()` — 116–164 moved wholesale. `prev_wk` keeps its
  `declare -A`; `samples_qbz` / `samples_wk_total` / `samples_total` are
  assigned without `local` so they survive into `report.sh`.
- `stats.sh`: see the behaviour note below.
- `report.sh`: `print_start_banner()`, `print_summary()`, `append_csv()`,
  `print_footer()`. `print_summary` and `append_csv` share `mean_qbz … max_total`
  as file-local globals — keeping them in the same file is why report is one
  file and not three.

### One dead-code wart to fix while splitting

`summarize` (167–180) both *prints* a human row and *echoes* a CSV row. Every
one of its four call sites (184, 185, 191, 195) either captures the output and
takes `| tail -1`, or redirects to `/dev/null` — so **the `printf` at line 178
never reaches a terminal**, and line 185 (`summarize "qbz" … >/dev/null`) is a
pure no-op that recomputes everything. The visible summary comes only from the
re-print block at 199–203.

The split should therefore make `stats.sh` hold a pure `summarize_csv()` that
echoes `mean,p50,p95,max` and prints nothing, and drop line 185. Output on
stdout is byte-identical; the CSV is unchanged. Flagging it explicitly because
"the split changed nothing observable" needs to be checkable, and this is the
one place where a reviewer diffing behaviour would otherwise get suspicious.

### Entry script after the split

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/measure-cpu/procstat.sh
source "$SCRIPT_DIR/lib/measure-cpu/procstat.sh"
# shellcheck source=scripts/lib/measure-cpu/discover.sh
source "$SCRIPT_DIR/lib/measure-cpu/discover.sh"
# shellcheck source=scripts/lib/measure-cpu/stats.sh
source "$SCRIPT_DIR/lib/measure-cpu/stats.sh"
# shellcheck source=scripts/lib/measure-cpu/sample.sh
source "$SCRIPT_DIR/lib/measure-cpu/sample.sh"
# shellcheck source=scripts/lib/measure-cpu/report.sh
source "$SCRIPT_DIR/lib/measure-cpu/report.sh"

resolve_targets
print_start_banner
sample_window
print_summary
append_csv
print_footer
```

`procstat.sh` is sourced before `discover.sh` only for readability — source
order is irrelevant here because nothing runs at source time; every lib is
function definitions only. State that in the README so nobody "fixes" the order
into a dependency graph that does not exist.

Globals crossing files: `LABEL`, `DURATION`, `LOG_CSV`, `CLK_TCK`, `QBZ_PID`,
`WEBKIT_PIDS`, `samples_*`, `mean_*`/`p50_*`/`p95_*`/`max_*`, `WINDOW`.
`local` inside functions: `candidates`, `pid`, `best_rss`, `best_pid`, `rss`,
`content`, `rest`, `logf`, `now_ts`, `dt`, `cur_qbz`, `d_qbz`, `pct_*`,
`wk_total`, `cur`, `d`, `n`, `sorted`, `mean`/`p50`/`p95`/`max`.
Careful with two that look local but are not: `prev_qbz` and `prev_ts` are
initialised at 117/123 and mutated inside the loop — if `sample_window` is one
function they can be `local` to it; they must not be split across two functions.

---

## 3. `scripts/slint-run.sh` (141) — argue for an exception

### Why is this file long?

Not multi-responsibility. Lines 45–141 are **96 lines of one linear build
pipeline**: pick a tier from `MemAvailable` → export the matching `RUSTFLAGS`/
`CARGO_*` → read the learned ETA → start a ticker → run one `cargo build` →
stop the ticker, record the duration → `exec` the binary. Each step consumes
what the previous step set (`avail_mb` → `TIER` → `eta_file` → `eta_secs` →
`build_start` → `tick_pid` → `build_secs`), and the steps are not
independently callable or independently useful.

The overage comes from somewhere else: **lines 2–44 are a 43-line comment
header** — the cargo-vs-direct-exec rationale, the MEMORY WALL story, the tier
table, the ticker explanation, and the usage block. That is documentation
living in a script, and most of it is *already duplicated* in `README.md:407–430`
(same tier table, same env-var examples).

### The recommended change: move the prose, not the code

1. Create `scripts/README.md` (which the project rules want anyway — a README
   per package explaining the WHY of each module) and move lines 2–38 into a
   `## slint-run.sh` section there, verbatim: the cargo-context rationale, the
   MEMORY WALL paragraph, the tier table with the not-byte-identical caveat,
   and the learned-ETA/ticker explanation.
2. Keep in the script an ~8-line header: the one-line summary, the `Usage:`
   block (39–44, verbatim — it is the fastest reference at the terminal), and
   `# Why the tiers exist, and the memory-wall history: scripts/README.md`.

**141 − 43 + 8 = ~106 lines.** Under budget, with zero risk to a CI contract,
and the pipeline stays readable top-to-bottom in one screen.

### Why not split the code

If the code were split, the natural cut is `lib/slint-run/tier.sh`
(53–74: `avail_mb`, the tier `if/elif/else`, the four `export`s) and
`lib/slint-run/progress.sh` (49–51 colours + `fmt_dur`, 76–89 ETA + start
banner, 91–110 ticker start, 129–135 ticker stop + banner). That is technically
possible and is written down here in case a reviewer insists — but it costs:

- The ticker is a **backgrounded subshell whose PID (`tick_pid`) is captured by
  a `trap … EXIT` installed at line 109 and removed by `trap - EXIT` at 131.**
  Splitting start and stop into two files puts an `EXIT` trap's installation and
  its removal in different files while the thing they protect (a background
  process) lives in a third scope. That is the single most breakable construct
  in the file, and moving it buys nothing.
- The subshell reads `build_start` and `eta_secs` from the parent at expansion
  time; those must stay non-`local`, and the constraint is only obvious while
  the code is adjacent.
- The tier block writes six globals consumed 60 lines later. Behind a
  `pick_tier` function they become invisible outputs.

So: **`slint-run.sh` gets a documented exception from the code-split rule and
gets under budget by relocating documentation.** If a future change pushes the
*executable* part past ~120 lines, revisit with the `tier.sh` cut above, which
is the safe half.

### If the split is done anyway — the one mandatory ordering note

Line 46 currently does `cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."`,
i.e. it computes the script dir and immediately discards it. Any `source` must
capture it first:

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."
# shellcheck source=scripts/lib/slint-run/tier.sh
source "$SCRIPT_DIR/lib/slint-run/tier.sh"
```

Sourcing with a path relative to the *current directory* after that `cd` would
work today only because the `cd` target happens to be the repo root — and would
break the moment the script is invoked from elsewhere. Use `$SCRIPT_DIR`.

---

## Also worth doing (not required by the budget)

`scripts/README.md` should cover all 12 scripts, one paragraph each, since three
of them (`slint-dev.sh`, `slint-dev-fast.sh`, `slint-run.sh`) differ in ways only
their headers currently explain, and `prune-incremental.sh:6,13` already
cross-references `slint-run.sh` in prose.

---

## Risks

**A sourcing mistake fails at runtime, and can fail only in CI.**
`source "$SCRIPT_DIR/lib/foo.sh"` on a missing or misspelled path prints
`line N: …/lib/foo.sh: No such file or directory` and, under `set -e`, exits
**1** immediately. For `slint-run.sh` that surfaces in
`build-slint-selfhosted.yml` as a red step *before* any cargo output, which is
at least loud. The dangerous variant is subtler: if `SCRIPT_DIR` is computed
after a `cd`, or with `$0` instead of `${BASH_SOURCE[0]}`, the script works when
run as `./scripts/slint-run.sh` from the repo root (what everyone tests locally
and what CI does) and breaks only for `bash scripts/slint-run.sh` from another
directory, or via a symlink. Test all three invocation forms before committing.

**Forgetting to `git add` a lib file.** The entry script keeps running fine on
the author's machine; CI checks out a tree without the lib and dies at source
time. This is the most likely real failure of this whole plan. Mitigation:
after the split, `git stash -u && ./scripts/slint-run.sh` (or a clean clone into
`/tmp`) before pushing.

**A top-level command in a lib.** Under `errexit` inherited from the entry, any
failing top-level line in a sourced file aborts the entry script *at source
time* — before `trap cleanup EXIT` is installed in `qbzd-acceptance.sh`, which
means a leaked `$SCRATCH` directory and possibly a running daemon on the test
port. Enforce the "function definitions only" rule in the lib README.

**`set -u` and array expansion.** `measure-cpu-window.sh` already tiptoes around
this with `"${WEBKIT_PIDS[@]:-}"` (lines 119, 144). Moving the array's producer
(`discover.sh`) away from its consumers (`sample.sh`) makes it easy for a later
edit to drop the `:-` and get `unbound variable` when no WebKit process is
running — the exact case the guard exists for, and one that never reproduces on
a developer box with the GUI open.

**Trap ownership in `qbzd-acceptance.sh`.** `cleanup` moves to `daemon.sh` but
`trap cleanup EXIT` stays in the entry, and it must stay after
`setup_scratch_env`. If someone later "tidies" the trap into `daemon.sh`'s top
level, it fires with `$SCRATCH` unset.

**Ordering dependencies between the extracted `check_*` phases.**
`check_daemon_down_codes` stops the daemon; `check_instance_lock` restarts it.
Once each is a named function, the list in the entry script *looks*
order-independent and is not. This is a review/readability risk, not a runtime
one, and the mitigation is a comment in the entry script plus the lib README.

**Doc drift (`slint-run.sh`).** Moving the tier table out of the script header
puts the third copy of it in the repo (`README.md`, `scripts/README.md`, the
tier code itself). If a threshold changes, all three must change. Reduce it to
two by having `scripts/README.md` link to `README.md:407` rather than restating
the table.
