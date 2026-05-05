#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
manifest="$repo_root/xdg-configs/tmux/mux/Cargo.toml"
bin="$repo_root/xdg-configs/tmux/mux/target/release/mux"
tmux_bin="$(command -v tmux)"

cargo build --release --manifest-path="$manifest" >/dev/null

socket="mux-sidebar-window-switch-$$"
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

"$tmux_bin" -L "$socket" -f /dev/null new-session -d -s mux-window-switch -x 120 -y 40 "sleep 120"
"$tmux_bin" -L "$socket" new-window -d -t mux-window-switch:2 "sleep 120"
PATH="$wrapper_dir:$PATH" HOME="$tmp_home" "$bin" sidebar open
sleep 1

PATH="$wrapper_dir:$PATH" HOME="$tmp_home" "$bin" window 2

current="$("$tmux_bin" -L "$socket" display-message -p '#{window_index}')"
if [[ "$current" != "2" ]]; then
    echo "expected to switch to window 2, got $current" >&2
    exit 1
fi

format=$'#{pane_id}\t#{@mux_sidebar}\t#{@mux_sidebar_token}\t#{pane_current_command}'
pane=""
for _ in {1..16}; do
    while IFS=$'\t' read -r candidate marker token command; do
        if [[ "$marker" == "1" && "$token" == "mux-sidebar-v1" && "$command" == "mux" ]]; then
            pane="$candidate"
            break
        fi
    done < <("$tmux_bin" -L "$socket" list-panes -F "$format")
    if [[ -n "$pane" ]]; then
        break
    fi
    sleep 0.1
done
if [[ -z "$pane" ]]; then
    echo "target window has no marked mux sidebar pane after mux window switch" >&2
    "$tmux_bin" -L "$socket" list-panes -a -F '#{window_index}:#{pane_id}:#{@mux_sidebar}:#{@mux_sidebar_token}:#{pane_current_command}' >&2
    exit 1
fi

nonspace=0
for _ in {1..16}; do
    nonspace="$("$tmux_bin" -L "$socket" capture-pane -p -t "$pane" | tr -d '[:space:]' | wc -c | tr -d ' ')"
    if [[ "$nonspace" -gt 0 ]]; then
        break
    fi
    sleep 0.1
done
if [[ "$nonspace" -eq 0 ]]; then
    echo "target window sidebar is blank after mux window switch" >&2
    "$tmux_bin" -L "$socket" capture-pane -p -t "$pane" >&2 || true
    exit 1
fi

echo "✓ sidebar window switch produced target sidebar with $nonspace non-space bytes"
