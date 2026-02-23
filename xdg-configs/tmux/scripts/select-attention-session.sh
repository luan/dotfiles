#!/usr/bin/env bash
# Switch to the first session with @attention set to "1"
while IFS= read -r session; do
  attn=$(tmux show-option -t "$session" -qv @attention 2>/dev/null)
  if [ "$attn" = "1" ]; then
    tmux switch-client -t "$session"
    exit 0
  fi
done < <(~/.config/tmux/scripts/session-order.sh)
exit 0
