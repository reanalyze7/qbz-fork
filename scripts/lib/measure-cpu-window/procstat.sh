# measure-cpu-window.sh — /proc readers. Pure-ish I/O: they read the
# filesystem and echo a value, they never touch the sampler's state.
# Sourced by scripts/measure-cpu-window.sh. Function definitions only.

read_jiffies() {
  # Field 14 (utime) + field 15 (stime) of /proc/PID/stat. Pre-paren tokens
  # in field 2 (comm) can contain spaces, so split off comm carefully.
  local pid="$1" stat content rest
  [[ -r "/proc/$pid/stat" ]] || { echo 0; return; }
  content="$(cat "/proc/$pid/stat" 2>/dev/null || echo "")"
  [[ -z "$content" ]] && { echo 0; return; }
  rest="${content#* (}"
  rest="${rest#*) }"
  read -r _state _ppid _pgrp _sid _tty _tpgid _flags _minflt _cminflt _majflt _cmajflt utime stime _rest <<< "$rest"
  echo $(( utime + stime ))
}

read_window_size() {
  # Best-effort: read the persisted window size from the qbz log. Not used in
  # math, just printed as context in the CSV.
  local logf="$HOME/.local/share/qbz/qbz.log"
  if [[ -r "$logf" ]]; then
    grep -oE 'size=[0-9]+x[0-9]+' "$logf" 2>/dev/null | tail -1 | cut -d= -f2 || echo "unknown"
  else
    echo "unknown"
  fi
}
