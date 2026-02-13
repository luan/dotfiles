#!/bin/bash
# Update session color and status bar immediately
session=$(tmux display-message -p '#S')

color=$(~/.config/tmux/scripts/session-color.sh "$session")

# Patch window format on first run, then set color + status in one shot
fmt=$(tmux show -gv window-status-current-format 2>/dev/null)
if [[ "$fmt" == *"@thm_mauve"* ]]; then
  tmux set -g window-status-current-format "${fmt//@thm_mauve/@session_color}"
fi

list=$(~/.config/tmux/scripts/session-list.sh)
tmux set -t "$session" @session_color "$color" \; set -g status-left " $list " \; refresh-client -S
