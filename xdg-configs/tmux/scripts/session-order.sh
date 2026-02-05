#!/bin/bash
# Returns sessions in custom order, appending any new sessions at the end
# Usage: session-order.sh [--all]
#   --all: include hidden sessions (default: filter them out)
ORDER_FILE="$HOME/.config/tmux/session-order"
HIDDEN_FILE="$HOME/.config/tmux/session-hidden"
touch "$ORDER_FILE" "$HIDDEN_FILE"

include_hidden=false
[ "$1" = "--all" ] && include_hidden=true

# Get all current sessions
current=$(tmux list-sessions -F '#S')

# Build ordered list
ordered=""
while read -r s; do
  [ -z "$s" ] && continue
  if echo "$current" | grep -qxF "$s"; then
    ordered="$ordered$s"$'\n'
  fi
done < "$ORDER_FILE"

# Append any sessions not in order file
while read -r s; do
  if ! grep -qxF "$s" "$ORDER_FILE"; then
    ordered="$ordered$s"$'\n'
    echo "$s" >> "$ORDER_FILE"
  fi
done <<< "$current"

# Filter hidden sessions unless --all
if [ "$include_hidden" = "false" ]; then
  filtered=""
  while read -r s; do
    [ -z "$s" ] && continue
    grep -qxF "$s" "$HIDDEN_FILE" || filtered="$filtered$s"$'\n'
  done <<< "$ordered"
  ordered="$filtered"
fi

printf '%s' "$ordered" | sed '/^$/d'
