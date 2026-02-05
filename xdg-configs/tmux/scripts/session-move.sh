#!/usr/bin/env bash
# Move a session up or down in custom order
# Usage: session-move.sh up|down [session_name]
ORDER_FILE="$HOME/.config/tmux/session-order"
direction=$1
current=${2:-$(tmux display-message -p '#S')}

# Read order into array
sessions=()
while IFS= read -r line; do
  [ -n "$line" ] && sessions+=("$line")
done < <(~/.config/tmux/scripts/session-order.sh --all)

# Find current session index
idx=-1
for i in "${!sessions[@]}"; do
  if [ "${sessions[$i]}" = "$current" ]; then
    idx=$i
    break
  fi
done

[ $idx -eq -1 ] && exit 1

# Calculate new position
if [ "$direction" = "up" ] && [ $idx -gt 0 ]; then
  new_idx=$((idx - 1))
elif [ "$direction" = "down" ] && [ $idx -lt $((${#sessions[@]} - 1)) ]; then
  new_idx=$((idx + 1))
else
  exit 0
fi

# Swap
tmp="${sessions[$idx]}"
sessions[$idx]="${sessions[$new_idx]}"
sessions[$new_idx]="$tmp"

# Write back
printf '%s\n' "${sessions[@]}" > "$ORDER_FILE"
tmux refresh-client -S
