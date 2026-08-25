# measure-cpu-window.sh — statistics. `summarize` is PURE: given a series
# it emits one CSV line, nothing else. It used to also print a
# human-readable row, but every call site discarded that with `tail -1`
# and the report table below is what the user actually sees — so the
# printing lives in one place now.
# Sourced by scripts/measure-cpu-window.sh. Function definitions only.

summarize() {
  local name="$1"
  shift
  local n="$#"
  local sorted
  sorted="$(printf '%s\n' "$@" | sort -n)"
  local mean p50 p95 max
  mean="$(printf '%s\n' "$@" | awk '{s+=$1} END{ if (NR>0) printf "%.1f", s/NR; else print 0 }')"
  p50="$(printf '%s\n' "$sorted" | awk -v n="$n" 'NR==int(n*0.50)+1 || (NR==1 && n==1) {print; exit}')"
  p95="$(printf '%s\n' "$sorted" | awk -v n="$n" 'NR==int(n*0.95)+1 || (NR==n) {print; exit}')"
  max="$(printf '%s\n' "$sorted" | tail -1)"
  echo "$mean,$p50,$p95,$max"
}

# The human-readable summary table. Takes the already-computed values so
# it does no arithmetic of its own.
print_summary_table() {
  printf '  %-10s mean=%5s%%  p50=%5s%%  p95=%5s%%  max=%5s%%\n' "qbz"    "$mean_qbz"    "$p50_qbz"    "$p95_qbz"    "$max_qbz"
  printf '  %-10s mean=%5s%%  p50=%5s%%  p95=%5s%%  max=%5s%%\n' "webkit" "$mean_wk"     "$p50_wk"     "$p95_wk"     "$max_wk"
  printf '  %-10s mean=%5s%%  p50=%5s%%  p95=%5s%%  max=%5s%%\n' "total"  "$mean_total"  "$p50_total"  "$p95_total"  "$max_total"
}
