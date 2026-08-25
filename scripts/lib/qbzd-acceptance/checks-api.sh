# qbzd acceptance — the checks that run against a LIVE daemon.
# Sourced by scripts/qbzd-acceptance.sh. Function definitions only.
#
# Every function here assumes start_daemon has already succeeded and leaves the
# daemon running; the lifecycle checks (checks-lifecycle.sh) are the ones that
# stop and restart it, which is why they run last.

check_boot() {
  echo "== isolated-root boot (env-driven scratch XDG) =="
  start_daemon
  echo "  scratch root: $SCRATCH"
}

check_exit_codes() {
  echo "== exit-code table (02 section 1.3): version ok, unknown verb is usage error =="
  qbzd version >/dev/null || fail "version != 0"
  set +e
  qbzd bogus-command-xyz >/dev/null 2>&1
  local rc_usage=$?
  set -e
  [ "$rc_usage" -eq 2 ] || fail "usage error != 2 (got $rc_usage)"
}

check_unknown_key_warning() {
  echo "== unknown-key config warning (01 section 10.2) =="
  grep -q 'unknown key: acceptance_script_marker' "$LOGFILE" \
    || fail "boot did not warn about the unrecognized qbzd.toml key -- see $LOGFILE"
}

check_status_shape() {
  echo "== status answers 'why is it silent' in one call (02 section 3.3.3) =="
  # This scratch daemon never logs in, so status is expected to report
  # auth.state=needs_auth and the CLI exits 4 (02 section 1.3: "status exits
  # nonzero on degraded state") -- that is the down-vs-unhealthy distinction
  # working correctly, not a script failure, so capture rc separately from
  # `set -e` and assert the SPECIFIC expected code rather than just >/dev/null.
  set +e
  local status_json
  status_json=$(qbzd status --json)
  local rc_status_needs_auth=$?
  set -e
  [ "$rc_status_needs_auth" -eq 4 ] || fail "status --json on an unauthenticated daemon != 4 (got $rc_status_needs_auth)"
  echo "$status_json" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for k in ('auth', 'audio', 'playback', 'qconnect', 'network', 'last_errors', 'driver_tick_age_ms'):
    assert k in d, f'missing status key: {k}'
assert d['auth']['state'] == 'needs_auth', d['auth']
"
}

check_ping_info_shape() {
  echo "== ping/info shape (02 section 3.3.1 / 3.3.2) =="
  curl -fsS "127.0.0.1:${PORT}/api/ping" | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d.get('ok') is True, d
assert d.get('app') == 'qbzd', d
assert 'api_version' in d, d
"
  curl -fsS "127.0.0.1:${PORT}/api/info" | python3 -c "
import json, sys
d = json.load(sys.stdin)
for k in ('app', 'version', 'api_version', 'bind', 'uptime_secs', 'data_root'):
    assert k in d, f'missing info key: {k}'
"
}

check_config_show() {
  echo "== config show --json matches the on-disk port (02 section 2.2 config verb) =="
  local cfg_port
  cfg_port=$(qbzd config show --json | python3 -c "import json,sys; print(json.load(sys.stdin)['server']['port'])")
  [ "$cfg_port" = "$PORT" ] || fail "config show --json port ($cfg_port) != expected test port ($PORT)"
}

check_export_import_roundtrip() {
  echo "== export/import roundtrip is a no-op (04 section 7) =="
  # The 04 section 7 roundtrip guarantee assumes a daemon "legitimately running"
  # its own settings -- concretely, one that has already moved past the fresh
  # store's quality_fallback_behavior=ask default (the TUI never writes "ask",
  # 03 section 3.3.2; "settings set" rejects it outright, cli/settings.rs). A
  # virgin store that never ran `qbzd setup`/`settings set` still holds "ask",
  # which the section 5.5 mapping unconditionally reports as `adapted` -- so
  # prime it first, exactly like a real first-run setup would, before proving
  # the no-op invariant.
  qbzd settings set audio.quality_fallback_behavior always_fallback >/dev/null
  local bundle="$SCRATCH/rt.qbzb"
  qbzd settings export "$bundle" >/dev/null
  local out
  out=$(qbzd settings import "$bundle" --dry-run)
  echo "$out" | grep -q "adapted (0)" || fail "roundtrip produced adaptations (no-change short-circuit broken -- 04 section 5.3 step 4)"
  echo "$out" | grep -qE '^applied \([0-9]+\)' || fail "roundtrip import produced no applied-bucket summary"
  rm -f "$bundle"
}

check_route_budget() {
  echo "== route budget: unknown route 404s, ping open, Origin rejected (02 section 3.1.2 / 3.1.4) =="
  curl -fsS "127.0.0.1:${PORT}/api/ping" >/dev/null || fail "open ping"
  local code ocode
  code=$(curl -s -o /dev/null -w '%{http_code}' "127.0.0.1:${PORT}/api/nope")
  [ "$code" = "404" ] || fail "unknown route: $code"
  ocode=$(curl -s -o /dev/null -w '%{http_code}' -H 'Origin: http://x' "127.0.0.1:${PORT}/api/status")
  [ "$ocode" = "403" ] || fail "origin shield: $ocode"
}
