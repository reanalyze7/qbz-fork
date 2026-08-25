#!/usr/bin/env bash
# QBZ Slint — build with cargo, then run the BINARY DIRECTLY.
#
# The rationale (why not `cargo run`, the memory-wall tiering, the progress
# ticker) and the full env-var reference live in scripts/README.md. It was
# 43 lines of prose in this header, which is what put the file over the
# 130-line budget; the code below is one linear build pipeline and is not
# safely splittable (the ticker pid is captured by a trap installed mid-way).
#
# Usage: ./scripts/slint-run.sh [extra app args]
#   FAST=1 | THREADS= CODEGEN_UNITS= OPT= | CAPPED=0 | NORUN=1 | NO_TICKER=1
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."

# --- Pretty helpers ----------------------------------------------------------
if [[ -t 2 ]]; then C_DIM=$'\033[2m'; C_BOLD=$'\033[1m'; C_GRN=$'\033[32m'; C_RST=$'\033[0m'
else C_DIM=""; C_BOLD=""; C_GRN=""; C_RST=""; fi
fmt_dur() { local s=$1; printf '%dm %02ds' $(( s / 60 )) $(( s % 60 )); }

avail_mb=$(free -m | awk '/^Mem:/ {print $7}')

# --- Pick build settings from available RAM (any knob overridable via env) ----
if [[ "${FAST:-0}" == 1 ]] || (( avail_mb >= 26000 )); then
  TIER=FAST;  THREADS="${THREADS:-16}"; CODEGEN_UNITS="${CODEGEN_UNITS:-16}";  OPT="${OPT:-3}"
  CAPPED="${CAPPED:-0}"          # ample RAM → earlyoom is the net, no cgroup cap
elif (( avail_mb >= 14000 )); then
  TIER=SAFE;  THREADS="${THREADS:-2}";  CODEGEN_UNITS="${CODEGEN_UNITS:-256}"; OPT="${OPT:-3}"
  CAPPED="${CAPPED:-1}"
else
  TIER=MIN;   THREADS="${THREADS:-1}";  CODEGEN_UNITS="${CODEGEN_UNITS:-256}"; OPT="${OPT:-2}"
  CAPPED="${CAPPED:-1}"
  echo "[slint-run] WARNING: only ${avail_mb} MB free — lowest-memory tier (slow). Close apps / drop to a TTY for a faster build." >&2
fi

# No x86 target-features here or in .cargo/config.toml (#549): the aes crate
# runtime-dispatches to AES-NI at identical speed, and compile-time features
# SIGILL older CPUs.
export RUSTFLAGS="-C link-arg=-fuse-ld=mold -Z threads=${THREADS}"
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CODEGEN_UNITS}"
export CARGO_PROFILE_RELEASE_OPT_LEVEL="${OPT}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"   # one rustc at a time (memory)

# --- Learned ETA: last successful build duration for THIS tier ---------------
eta_dir="${XDG_CACHE_HOME:-$HOME/.cache}/qbz-slint"
eta_file="${eta_dir}/last-build-${TIER}.secs"
eta_secs=0
[[ -r "${eta_file}" ]] && eta_secs=$(cat "${eta_file}" 2>/dev/null || echo 0)
[[ "${eta_secs}" =~ ^[0-9]+$ ]] || eta_secs=0

echo "[slint-run] tier=${TIER} avail=${avail_mb}MB → threads=${THREADS} codegen-units=${CODEGEN_UNITS} opt-level=${OPT} capped=${CAPPED}"

# --- Start banner ------------------------------------------------------------
build_start=$(date +%s)
if (( eta_secs > 0 )); then eta_txt="~$(fmt_dur "${eta_secs}") (last ${TIER})"; else eta_txt="unknown (first ${TIER} build)"; fi
printf '%s[slint-run] ▶ build started %s  ·  ETA %s%s\n' \
  "${C_BOLD}" "$(date '+%H:%M:%S')" "${eta_txt}" "${C_RST}"

# --- Live ticker: elapsed / ETA / percent, every 15s while codegen is silent -
tick_pid=""
if [[ "${NO_TICKER:-0}" != 1 ]] && [[ -t 2 ]]; then
  (
    while true; do
      sleep 15
      now=$(date +%s); el=$(( now - build_start ))
      if (( eta_secs > 0 )); then
        pct=$(( el * 100 / eta_secs )); (( pct > 99 )) && pct=99
        printf '%s[slint-run] ⏱  %s elapsed · ETA ~%s · ~%d%%%s\n' \
          "${C_DIM}" "$(fmt_dur "${el}")" "$(fmt_dur "${eta_secs}")" "${pct}" "${C_RST}" >&2
      else
        printf '%s[slint-run] ⏱  %s elapsed%s\n' "${C_DIM}" "$(fmt_dur "${el}")" "${C_RST}" >&2
      fi
    done
  ) &
  tick_pid=$!
  # Safety net: if the build aborts (set -e), don't leak the ticker.
  trap '[[ -n "${tick_pid}" ]] && kill "${tick_pid}" 2>/dev/null || true' EXIT
fi

# --- The build ---------------------------------------------------------------
if [[ "${CAPPED}" == 1 ]] && command -v cargo-capped >/dev/null 2>&1; then
  # Cap the cgroup so the GLOBAL MemAvailable floor never reaches earlyoom's
  # 10% trigger (~3.2 GB on this box). Empirically (2026-07-04, ftrace-caught):
  # a 3.5 GB margin is NOT enough — the desktop grows a few GB during an
  # hour-long build, avail cratered to 3.1 GB and earlyoom SIGTERM'd rustc
  # while the cgroup sat comfortably under its cap. memory.high is the anchor
  # (throttle-to-swap early); leave ~9 GB of global headroom above the trigger.
  high=$(( avail_mb - 9000 )); (( high > 24000 )) && high=24000; (( high < 8000 )) && high=8000
  export BUILD_MEM_HIGH="${high}M"
  export BUILD_MEM_MAX="$(( high + 2000 ))M"
  echo "[slint-run] cgroup cap: high=${BUILD_MEM_HIGH} max=${BUILD_MEM_MAX}"
  cargo-capped cargo +nightly build --release --manifest-path crates/Cargo.toml -p qbz
else
  cargo +nightly build --release --manifest-path crates/Cargo.toml -p qbz
fi

# --- Stop the ticker, record the duration, print the final banner ------------
[[ -n "${tick_pid}" ]] && { kill "${tick_pid}" 2>/dev/null || true; wait "${tick_pid}" 2>/dev/null || true; }
trap - EXIT
build_secs=$(( $(date +%s) - build_start ))
mkdir -p "${eta_dir}" 2>/dev/null && printf '%s\n' "${build_secs}" > "${eta_file}" 2>/dev/null || true
printf '%s[slint-run] ✔ build finished %s  ·  took %s  (tier %s)%s\n' \
  "${C_BOLD}${C_GRN}" "$(date '+%H:%M:%S')" "$(fmt_dur "${build_secs}")" "${TIER}" "${C_RST}"

[[ "${NORUN:-0}" == 1 ]] && { echo "[slint-run] build done (NORUN set)."; exit 0; }

# exec the binary directly — no `cargo run`, so no CARGO_* env / cargo context,
# so the monitor shows `qbz-slint`.
exec crates/target/release/qbz "$@"
