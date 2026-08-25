# qbzd acceptance — the checks that manipulate daemon lifecycle.
# Sourced by scripts/qbzd-acceptance.sh. Function definitions only.
#
# ORDER IS LOAD-BEARING and the entry script must not reorder these:
# check_daemon_down_codes STOPS the daemon, and check_instance_lock is what
# starts it again. Running the API checks after these would hit a dead daemon.

check_daemon_down_codes() {
  echo "== daemon-down: ping/status exit 3 (02 section 1.3) =="
  stop_daemon
  set +e
  qbzd ping >/dev/null 2>&1
  local rc_ping=$?
  qbzd status >/dev/null 2>&1
  local rc_status=$?
  set -e
  [ "$rc_ping" -eq 3 ]   || fail "ping daemon-down != 3 (got $rc_ping)"
  [ "$rc_status" -eq 3 ] || fail "status daemon-down != 3 (got $rc_status)"
}

check_instance_lock() {
  echo "== instance lock: a second 'qbzd run' on the same root exits 3 (01 section 8.3) =="
  start_daemon
  set +e
  timeout 5 "$QBZD_BIN" run >>"$LOGFILE" 2>&1
  local rc_second=$?
  set -e
  [ "$rc_second" -eq 3 ] || fail "second 'qbzd run' on the same data root != 3 (got $rc_second)"
  qbzd ping >/dev/null || fail "the first daemon stopped answering after the double-start attempt"
}

check_non_tty_setup() {
  echo "== non-tty 'qbzd setup' exits 2, never hangs (03 section 2.4) =="
  set +e
  timeout 5 "$QBZD_BIN" setup </dev/null >/dev/null 2>&1
  local rc_setup=$?
  set -e
  [ "$rc_setup" -eq 2 ] || fail "non-tty setup != 2 (got $rc_setup)"
}
