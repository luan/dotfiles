#!/bin/bash
# Toggle a session's hidden state
HIDDEN_FILE="$HOME/.config/tmux/session-hidden"
touch "$HIDDEN_FILE"

session="$1"
[ -z "$session" ] && exit 1

if grep -qxF "$session" "$HIDDEN_FILE"; then
  # Remove from hidden
  grep -vxF "$session" "$HIDDEN_FILE" > "$HIDDEN_FILE.tmp"
  mv "$HIDDEN_FILE.tmp" "$HIDDEN_FILE"
else
  # Add to hidden
  echo "$session" >> "$HIDDEN_FILE"
fi
