#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
manifest="$repo_root/xdg-configs/tmux/mux/Cargo.toml"
bin="$repo_root/xdg-configs/tmux/mux/target/release/mux"
out_dir="${1:-$repo_root/docs/artifacts/mux-sidebar-live-memory-$(date -u +%Y%m%dT%H%M%SZ)}"
sample_seconds="${MUX_SIDEBAR_MEMORY_SAMPLE_SECONDS:-8}"
shapes="${MUX_SIDEBAR_MEMORY_SHAPES:-small medium large}"
tmux_bin="$(command -v tmux)"

mkdir -p "$out_dir"
if [[ -n "${MUX_SIDEBAR_CARGO_FEATURES:-}" ]]; then
  cargo build --release --manifest-path "$manifest" --features "$MUX_SIDEBAR_CARGO_FEATURES" >/dev/null
else
  cargo build --release --manifest-path "$manifest" >/dev/null
fi

run_tmux() {
  command tmux -L "$socket_name" "$@"
}

create_shape() {
  local sessions="$1"
  local panes_per_session="$2"
  local panes_per_window=5

  for s in $(seq 1 "$sessions"); do
    local name
    name="$(printf 'muxmem-%02d' "$s")"
    run_tmux new-session -d -x 220 -y 80 -s "$name" "sleep 3600"
    run_tmux set-option -t "$name" -q @sidebar_status "idle-$s"
    run_tmux set-option -t "$name" -q @sidebar_progress "$((s % 100))"
    local remaining="$panes_per_session"
    local window_idx=0
    local first_window
    first_window="$(run_tmux display-message -p -t "$name" '#{window_id}')"
    while [[ "$remaining" -gt 0 ]]; do
      local target="$first_window"
      local panes_this_window="$panes_per_window"
      if [[ "$remaining" -lt "$panes_this_window" ]]; then
        panes_this_window="$remaining"
      fi
      if [[ "$window_idx" -gt 0 ]]; then
        target="$(run_tmux new-window -P -F '#{window_id}' -d -t "$name" -n "w$window_idx" "sleep 3600")"
      fi
      for _ in $(seq 2 "$panes_this_window"); do
        run_tmux split-window -t "$target" -d "sleep 3600"
        run_tmux select-layout -t "$target" tiled >/dev/null 2>&1 || true
      done
      run_tmux select-layout -t "$target" tiled >/dev/null 2>&1 || true
      remaining=$((remaining - panes_this_window))
      window_idx=$((window_idx + 1))
    done
  done
}

record_shape() {
  local shape="$1"
  {
    echo "tmux version:"
    run_tmux -V
    echo
    echo "sessions:"
    run_tmux list-sessions -F '#{session_id}\t#{session_name}\twindows=#{session_windows}\tattached=#{session_attached}'
    echo
    echo "windows:"
    run_tmux list-windows -a -F '#{session_name}:#{window_index}\t#{window_id}\tpanes=#{window_panes}\tactive=#{window_active}\tname=#{window_name}'
    echo
    echo "panes:"
    run_tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index}\t#{pane_id}\tactive=#{pane_active}\tdead=#{pane_dead}\ttitle=#{pane_title}\tcmd=#{pane_current_command}'
  } > "$out_dir/$shape.tmux-shape.txt"
}

churn_statuses() {
  local sessions="$1"
  local iterations="$2"

  for i in $(seq 1 "$iterations"); do
    for s in $(seq 1 "$sessions"); do
      local name
      name="$(printf 'muxmem-%02d' "$s")"
      run_tmux set-option -t "$name" -q @sidebar_status "churn-$i-session-$s"
      run_tmux set-option -t "$name" -q @sidebar_progress "$(((i + s) % 100))"
      run_tmux set-option -t "$name" -q @attention "$(((i + s) % 7 == 0 ? 1 : 0))"
    done
    sleep 0.5
  done
}

sample_process() {
  local pid="$1"
  local shape="$2"
  local snapshot="$3"

  echo "second,rss_kb,vsz_kb,cpu_pct,snapshot_bytes" > "$out_dir/$shape.daemon-rss.csv"
  for second in $(seq 0 "$sample_seconds"); do
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    local ps_line snapshot_bytes
    ps_line="$(ps -o rss= -o vsz= -o %cpu= -p "$pid" | awk 'NF {print $1 "," $2 "," $3}')"
    snapshot_bytes=0
    if [[ -f "$snapshot" ]]; then
      snapshot_bytes="$(stat -f %z "$snapshot")"
    fi
    printf '%s,%s,%s\n' "$second" "$ps_line" "$snapshot_bytes" >> "$out_dir/$shape.daemon-rss.csv"
    sleep 1
  done
}

profile_shape() {
  local shape="$1"
  local sessions="$2"
  local panes_per_session="$3"
  local expected_panes="$4"

  socket_name="mux-sidebar-memory-$shape-$$"
  tmp_home="$(mktemp -d)"
  wrapper_dir="$(mktemp -d)"
  cat > "$wrapper_dir/tmux" <<EOF
#!/usr/bin/env bash
exec "$tmux_bin" -L "$socket_name" "\$@"
EOF
  chmod +x "$wrapper_dir/tmux"

  cleanup() {
    run_tmux kill-server >/dev/null 2>&1 || true
    rm -rf "$tmp_home" "$wrapper_dir"
  }
  trap cleanup RETURN

  create_shape "$sessions" "$panes_per_session"
  record_shape "$shape"

  local actual_panes
  actual_panes="$(run_tmux list-panes -a | wc -l | tr -d ' ')"
  if [[ "$actual_panes" -lt "$expected_panes" ]]; then
    echo "expected at least $expected_panes panes for $shape, got $actual_panes" >&2
    return 1
  fi

  PATH="$wrapper_dir:$PATH" HOME="$tmp_home" "$bin" sidebar-daemon > "$out_dir/$shape.daemon.stdout.txt" 2> "$out_dir/$shape.daemon.stderr.txt" &
  local daemon_pid="$!"
  local snapshot="$tmp_home/.local/state/mux/sidebar/snapshot.msgpack"

  churn_statuses "$sessions" "$sample_seconds" > "$out_dir/$shape.status-churn.stdout.txt" 2> "$out_dir/$shape.status-churn.stderr.txt" &
  local churn_pid="$!"

  sample_process "$daemon_pid" "$shape" "$snapshot"

  kill "$churn_pid" >/dev/null 2>&1 || true
  kill "$daemon_pid" >/dev/null 2>&1 || true
  wait "$churn_pid" >/dev/null 2>&1 || true
  wait "$daemon_pid" >/dev/null 2>&1 || true

  if [[ -f "$snapshot" ]]; then
    cp "$snapshot" "$out_dir/$shape.snapshot.msgpack"
  fi

  PATH="$wrapper_dir:$PATH" HOME="$tmp_home" /usr/bin/time -l "$bin" sidebar profile 4000 visible-idle > "$out_dir/$shape.visible-idle.json" 2> "$out_dir/$shape.visible-idle.time.txt"
  PATH="$wrapper_dir:$PATH" HOME="$tmp_home" /usr/bin/time -l "$bin" sidebar profile 4000 hidden-idle > "$out_dir/$shape.hidden-idle.json" 2> "$out_dir/$shape.hidden-idle.time.txt"
  PATH="$wrapper_dir:$PATH" HOME="$tmp_home" /usr/bin/time -l "$bin" sidebar profile 4000 active-animation > "$out_dir/$shape.active-animation.json" 2> "$out_dir/$shape.active-animation.time.txt"
}

{
  date -u '+%Y-%m-%dT%H:%M:%SZ'
  git -C "$repo_root" rev-parse HEAD
  git -C "$repo_root" status --short
  uname -a
  sw_vers 2>/dev/null || true
  rustc --version
  cargo --version
  tmux -V
  echo "sample_seconds=$sample_seconds"
  echo "shapes=$shapes"
} > "$out_dir/environment.txt"

for shape in $shapes; do
  case "$shape" in
    small) profile_shape small 1 5 5 ;;
    medium) profile_shape medium 5 10 50 ;;
    large) profile_shape large 20 15 300 ;;
    *) echo "unknown shape: $shape" >&2; exit 2 ;;
  esac
done

python3 - "$out_dir" <<'PY' > "$out_dir/summary.tsv"
import csv
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
print("shape\tsessions\tpanes\tmax_rss_kb\tmax_vsz_kb\tmax_cpu_pct\tmax_snapshot_bytes")
for shape in (root / "environment.txt").read_text().split("shapes=", 1)[1].splitlines()[0].split():
    sessions = panes = 0
    section = None
    for line in (root / f"{shape}.tmux-shape.txt").read_text().splitlines():
        if line == "sessions:":
            section = "sessions"
            continue
        if line == "windows:":
            section = "windows"
            continue
        if line == "panes:":
            section = "panes"
            continue
        if not line or line.endswith(":"):
            continue
        if section == "sessions" and ("\\t" in line or "\t" in line):
            sessions += 1
        elif section == "panes" and ("\\t" in line or "\t" in line):
            panes += 1
    rows = list(csv.DictReader((root / f"{shape}.daemon-rss.csv").open()))
    max_rss = max((int(r["rss_kb"]) for r in rows if r["rss_kb"]), default=0)
    max_vsz = max((int(r["vsz_kb"]) for r in rows if r["vsz_kb"]), default=0)
    max_cpu = max((float(r["cpu_pct"]) for r in rows if r["cpu_pct"]), default=0)
    max_snapshot = max((int(r["snapshot_bytes"]) for r in rows if r["snapshot_bytes"]), default=0)
    print(f"{shape}\t{sessions}\t{panes}\t{max_rss}\t{max_vsz}\t{max_cpu:.1f}\t{max_snapshot}")
PY

echo "wrote $out_dir"
