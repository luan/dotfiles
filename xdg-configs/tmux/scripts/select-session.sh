#!/usr/bin/env bash
# Select tmux session by index (1-based, using custom order)
index=$1
session=$(~/.config/tmux/scripts/session-order.sh | sed -n "${index}p")
[ -n "$session" ] && tmux switch-client -t "$session"
