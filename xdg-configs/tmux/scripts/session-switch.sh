#!/usr/bin/env bash
# Switch to prev/next session using custom order (skips hidden sessions)
dir=$1
current=$(tmux display-message -p '#S')
sessions=($(~/.config/tmux/scripts/session-order.sh))
count=${#sessions[@]}

[ "$count" -eq 0 ] && exit 0

for i in "${!sessions[@]}"; do
  if [ "${sessions[$i]}" = "$current" ]; then
    if [ "$dir" = "prev" ]; then
      idx=$(( (i - 1 + count) % count ))
    else
      idx=$(( (i + 1) % count ))
    fi
    tmux switch-client -t "${sessions[$idx]}"
    exit 0
  fi
done

# Current session not in list (hidden) - go to first or last visible session
if [ "$dir" = "prev" ]; then
  tmux switch-client -t "${sessions[$((count - 1))]}"
else
  tmux switch-client -t "${sessions[0]}"
fi
