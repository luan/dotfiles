#!/opt/homebrew/bin/bash
# Returns sessions in custom order, prepending any new sessions
# Usage: session-order.sh [--all]
#   --all: include hidden sessions (default: filter them out)
ORDER_FILE="$HOME/.config/tmux/session-order"
HIDDEN_FILE="$HOME/.config/tmux/session-hidden"
touch "$ORDER_FILE" "$HIDDEN_FILE"

include_hidden=false
[ "$1" = "--all" ] && include_hidden=true

# Load current sessions into associative array
current=$(tmux list-sessions -F '#S')
declare -A alive
while read -r s; do
  [ -n "$s" ] && alive[$s]=1
done <<< "$current"

# Load hidden sessions
declare -A hidden
if [ "$include_hidden" = "false" ]; then
  while read -r s; do
    [ -n "$s" ] && hidden[$s]=1
  done < "$HIDDEN_FILE"
fi

# Load order file into set for fast lookup
declare -A in_order
while read -r s; do
  [ -n "$s" ] && in_order[$s]=1
done < "$ORDER_FILE"

# Collect new sessions (not in order file)
new_sessions=""
while read -r s; do
  [ -n "$s" ] && [ -z "${in_order[$s]}" ] && new_sessions="$new_sessions$s"$'\n'
done <<< "$current"

# Prepend new sessions to order file
if [ -n "$new_sessions" ]; then
  tmp=$(mktemp)
  printf '%s' "$new_sessions" | cat - "$ORDER_FILE" > "$tmp" && mv "$tmp" "$ORDER_FILE"
fi

# Output: new sessions first, then ordered (alive + visible only)
result=""
if [ -n "$new_sessions" ]; then
  while read -r s; do
    [ -n "$s" ] && [ -z "${hidden[$s]}" ] && result="$result$s"$'\n'
  done <<< "${new_sessions%$'\n'}"
fi
while read -r s; do
  [ -z "$s" ] && continue
  [ -n "${alive[$s]}" ] && [ -z "${hidden[$s]}" ] && result="$result$s"$'\n'
done < "$ORDER_FILE"

printf '%s' "$result" | sed '/^$/d'
