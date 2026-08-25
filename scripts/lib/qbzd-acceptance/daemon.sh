# qbzd acceptance — daemon lifecycle: config writing, start/stop, and the
# escalating kill used by both stop_daemon and the EXIT trap.
# Sourced by scripts/qbzd-acceptance.sh. Function definitions only.
#
# DAEMON_PID is deliberately a global owned by the entry script: cleanup() runs
# from the EXIT trap and must see whatever start_daemon last set.

# Escalating kill helper: SIGTERM → poll 5s → SIGKILL → poll 2s.
# Returns 0 if confirmed dead, 1 if still alive after SIGKILL.
# If still alive after SIGKILL, prints a warning with the PID.
kill_and_confirm() {
  local pid=$1
  [ -n "$pid" ] || return 0

  # SIGTERM and poll for up to 5 seconds (25 x 0.2s).
  kill -TERM "$pid" 2>/dev/null || true
  for _ in $(seq 1 25); do kill -0 "$pid" 2>/dev/null || return 0; sleep 0.2; done

  # Still alive; escalate to SIGKILL and poll for up to 2 seconds (10 x 0.2s).
  kill -KILL "$pid" 2>/dev/null || true
  for _ in $(seq 1 10); do kill -0 "$pid" 2>/dev/null || return 0; sleep 0.2; done

  # Still alive after SIGKILL; print warning and return 1.
  echo "WARNING: qbzd (PID $pid) still alive after SIGKILL" >&2
  return 1
}

cleanup() {
  if [ -n "$DAEMON_PID" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill_and_confirm "$DAEMON_PID" || true
  fi
  rm -rf "$SCRATCH"
}

qbzd() { "$QBZD_BIN" --host "$QBZD_HOST" "$@"; }

write_config() {
  mkdir -p "$XDG_CONFIG_HOME/qbzd"
  # Pins the test port (never 8182 -- avoids colliding with a real daemon) and
  # plants one unrecognized key so the 01 §10.2 unknown-key startup warning is
  # exercised on every boot this script does.
  cat > "$XDG_CONFIG_HOME/qbzd/qbzd.toml" <<TOML
config_version = 1
acceptance_script_marker = "unknown key on purpose (01 section 10.2)"

[server]
bind = "127.0.0.1"
port = $PORT
TOML
}

start_daemon() {
  write_config
  : > "$LOGFILE"
  "$QBZD_BIN" run >>"$LOGFILE" 2>&1 &
  DAEMON_PID=$!
  for _ in $(seq 1 50); do
    qbzd ping >/dev/null 2>&1 && return 0
    kill -0 "$DAEMON_PID" 2>/dev/null || fail "daemon exited during boot -- see $LOGFILE"
    sleep 0.2
  done
  fail "daemon did not answer ping within 10s -- see $LOGFILE"
}

stop_daemon() {
  [ -n "$DAEMON_PID" ] || return 0
  if kill_and_confirm "$DAEMON_PID"; then
    DAEMON_PID=""
  else
    # Process still alive after escalation; leave DAEMON_PID set so cleanup() gets another chance.
    true
  fi
}
