#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
manifest="$repo_root/xdg-configs/tmux/mux/Cargo.toml"
bin="$repo_root/xdg-configs/tmux/mux/target/release/mux"
tmux_bin="$(command -v tmux)"

cargo build --release --manifest-path="$manifest" >/dev/null

socket="mux-sidebar-first-paint-$$"
tmp_home="$(mktemp -d)"
wrapper_dir="$(mktemp -d)"

cleanup() {
    "$tmux_bin" -L "$socket" kill-server >/dev/null 2>&1 || true
    sleep 0.1
    rm -rf "$tmp_home" "$wrapper_dir" >/dev/null 2>&1 || true
}
trap cleanup EXIT

printf '%s\n' '#!/usr/bin/env bash' "exec \"$tmux_bin\" -L \"$socket\" \"\$@\"" > "$wrapper_dir/tmux"
chmod +x "$wrapper_dir/tmux"

"$tmux_bin" -L "$socket" -f /dev/null new-session -d -s mux-first-paint -x 120 -y 40 "sleep 120"
pane="$("$tmux_bin" -L "$socket" split-window -h -b -f -l 40 -d -P -F '#{pane_id}' \
    "exec env PATH=\"$wrapper_dir:\$PATH\" HOME=\"$tmp_home\" MUX_SIDEBAR_TMUX=mux-sidebar-v1 MUX_SIDEBAR_MARKED=1 \"$bin\" sidebar")"
"$tmux_bin" -L "$socket" set-option -p -t "$pane" @mux_sidebar 1
"$tmux_bin" -L "$socket" set-option -p -t "$pane" @mux_sidebar_token mux-sidebar-v1

nonspace=0
for _ in {1..18}; do
    sleep 0.1
    nonspace="$("$tmux_bin" -L "$socket" capture-pane -p -t "$pane" | tr -d '[:space:]' | wc -c | tr -d ' ')"
    if [[ "$nonspace" -gt 0 ]]; then
        break
    fi
done
if [[ "$nonspace" -eq 0 ]]; then
    echo "sidebar first paint is blank" >&2
    "$tmux_bin" -L "$socket" capture-pane -p -t "$pane" >&2 || true
    exit 1
fi

echo "✓ sidebar first paint produced $nonspace non-space bytes"
