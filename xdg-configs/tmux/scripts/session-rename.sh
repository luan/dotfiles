#!/usr/bin/env bash
# Rename current tmux session and update order file to preserve position
set -eo pipefail

ORDER_FILE="$HOME/.config/tmux/session-order"
old_name=$(tmux display-message -p '#S')

printf "\033[33mRename session:\033[0m %s\n\n" "$old_name"
read -rep "New name: " -i "$old_name" new_name

# Trim whitespace, bail on empty or unchanged
new_name=$(echo "$new_name" | xargs)
[[ -z "$new_name" || "$new_name" == "$old_name" ]] && exit 0

# Check for collision
if tmux has-session -t "=$new_name" 2>/dev/null; then
  printf "\033[31mSession '%s' already exists\033[0m\n" "$new_name"
  read -rp "Press enter to exit..."
  exit 1
fi

tmux rename-session -t "=$old_name" "$new_name"

# Patch order file in-place so position is preserved
if [[ -f "$ORDER_FILE" ]]; then
  sed -i '' "s/^${old_name}$/${new_name}/" "$ORDER_FILE"
fi
