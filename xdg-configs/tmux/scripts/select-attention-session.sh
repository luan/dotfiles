#!/usr/bin/env bash
# Switch to the first session with @attention set to "1"
# Single tmux call, no file I/O
target=$(tmux list-sessions -F '#{@attention} #S' | awk '$1 == "1" { print $2; exit }')
[ -n "$target" ] && tmux switch-client -t "$target"
exit 0
