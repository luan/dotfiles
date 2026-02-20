#!/bin/bash
# Update session color and status bar immediately
session=$(tmux display-message -p '#S')

# Clear attention flag on the focused session
tmux set-option -t "$session" -u @attention 2>/dev/null

# Compute position for dynamic sessions so accent color matches status bar
case "$session" in
  claude|dotfiles)
    color=$(~/.config/tmux/scripts/session-color.sh "$session")
    ;;
  *)
    sessions=()
    while IFS= read -r s; do [ -n "$s" ] && sessions+=("$s"); done < <(~/.config/tmux/scripts/session-order.sh)
    dynamic_pos=0; dynamic_total=0
    for s in "${sessions[@]}"; do case "$s" in claude|dotfiles) ;; *) dynamic_total=$((dynamic_total + 1)) ;; esac; done
    for s in "${sessions[@]}"; do
      case "$s" in claude|dotfiles) continue ;; esac
      [ "$s" = "$session" ] && break
      dynamic_pos=$((dynamic_pos + 1))
    done
    color=$(~/.config/tmux/scripts/session-color.sh --pos "$dynamic_pos" --total "$dynamic_total" "$session")
    ;;
esac

# Patch window format on first run, then set color + status in one shot
fmt=$(tmux show -gv window-status-current-format 2>/dev/null)
if [[ "$fmt" == *"@thm_mauve"* ]]; then
  tmux set -g window-status-current-format "${fmt//@thm_mauve/@session_color}"
fi

list=$(~/.config/tmux/scripts/session-list.sh)
tmux set -t "$session" @session_color "$color" \; set -g status-left " $list " \; refresh-client -S
