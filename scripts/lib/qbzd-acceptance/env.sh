# qbzd acceptance — preflight and the isolated scratch profile root.
# Sourced by scripts/qbzd-acceptance.sh. Function definitions only: everything
# here writes into the ENTRY script's global scope on purpose, so nothing may
# be declared `local`.

fail() { echo "FAIL: $1" >&2; exit 1; }

require_tools() {
  command -v curl    >/dev/null 2>&1 || fail "curl is required"
  command -v python3 >/dev/null 2>&1 || fail "python3 is required (status/ping/info shape checks)"
  command -v timeout >/dev/null 2>&1 || fail "timeout (coreutils) is required"
  [ -x "$QBZD_BIN" ] || fail "qbzd binary not found/executable at $QBZD_BIN -- build it first (release, on its own: cargo build --release -p qbzd)"
}

# Isolated scratch profile root. NEVER the real daemon/desktop roots: dirs::
# config_dir()/data_dir()/cache_dir() (crates/qbzd/src/paths.rs) honor these
# three env vars on Linux, and every qbzd invocation (daemon and CLI alike)
# resolves its roots this same way (main.rs: ProfileRoots::resolve(None, None)
# everywhere) -- so exporting them here is sufficient isolation for the whole
# script, with no --config/--data-root flag needed.
#
# The caller must install `trap cleanup EXIT` AFTER calling this: cleanup
# rm -rf's $SCRATCH, which does not exist until this runs.
setup_scratch_env() {
  SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/qbzd-acceptance.XXXXXX")"
  export XDG_CONFIG_HOME="$SCRATCH/config"
  export XDG_DATA_HOME="$SCRATCH/data"
  export XDG_CACHE_HOME="$SCRATCH/cache"
  mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME"

  # Belt-and-braces: refuse to run if isolation somehow didn't take.
  case "$XDG_CONFIG_HOME" in
    "$HOME"/.config|"$HOME"/.config/*) fail "refusing to run: XDG_CONFIG_HOME resolved under \$HOME/.config -- isolation broke" ;;
  esac

  QBZD_HOST="127.0.0.1:$PORT"
  LOGFILE="$SCRATCH/qbzd.log"
  DAEMON_PID=""
}

# Never steal a live port -- if something already answers, stop rather than guess.
assert_port_free() {
  if curl -fsS -m 1 "127.0.0.1:${PORT}/api/ping" >/dev/null 2>&1; then
    fail "something is already answering on 127.0.0.1:${PORT} -- set QBZD_TEST_PORT to a free port"
  fi
}
