#!/usr/bin/env bash
# qbzd P0 acceptance -- scripted checks (05-implementation-plan.md T16).
#
# This file is the RUN ORDER and nothing else; the checks themselves live in
# scripts/lib/qbzd-acceptance/. See scripts/README.md for the safety story
# (prebuilt binary, isolated scratch XDG root, non-default port) and the env
# vars. The order below is not free -- see checks-lifecycle.sh.
#
# Usage:
#   ./scripts/qbzd-acceptance.sh
#   QBZD_BIN=/path/to/qbzd ./scripts/qbzd-acceptance.sh
#   QBZD_TEST_PORT=28182 ./scripts/qbzd-acceptance.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$ROOT/scripts/lib/qbzd-acceptance"
# shellcheck source=lib/qbzd-acceptance/env.sh
. "$LIB/env.sh"
# shellcheck source=lib/qbzd-acceptance/daemon.sh
. "$LIB/daemon.sh"
# shellcheck source=lib/qbzd-acceptance/checks-api.sh
. "$LIB/checks-api.sh"
# shellcheck source=lib/qbzd-acceptance/checks-lifecycle.sh
. "$LIB/checks-lifecycle.sh"

QBZD_BIN="${QBZD_BIN:-$ROOT/crates/target/release/qbzd}"
PORT="${QBZD_TEST_PORT:-28182}"

require_tools
setup_scratch_env
# AFTER setup_scratch_env: cleanup rm -rf's $SCRATCH, which does not exist yet.
trap cleanup EXIT
assert_port_free

check_boot
check_exit_codes
check_unknown_key_warning
check_status_shape
check_ping_info_shape
check_config_show
check_export_import_roundtrip
check_route_budget

# These stop and restart the daemon; keep them last and in this order.
check_daemon_down_codes
check_instance_lock
check_non_tty_setup

stop_daemon
echo "ALL SCRIPTED CHECKS PASSED"
