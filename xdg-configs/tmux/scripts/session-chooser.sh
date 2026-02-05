#!/bin/bash
# Session chooser with fzf - supports alt-h to toggle hidden
SCRIPTS_DIR="$HOME/.config/tmux/scripts"
ORDER_FILE="$HOME/.config/tmux/session-order"
HIDDEN_FILE="$HOME/.config/tmux/session-hidden"
touch "$ORDER_FILE" "$HIDDEN_FILE"

build_list() {
  local GRAY=$'\033[90m'
  local YELLOW=$'\033[33m'
  local RESET=$'\033[0m'

  current=$(tmux display-message -p '#S')
  all_sessions=$(tmux list-sessions -F '#S')

  # Build ordered list (include all sessions)
  sessions=""
  while IFS= read -r s; do
    [ -z "$s" ] && continue
    echo "$all_sessions" | grep -qxF "$s" && sessions="$sessions$s"$'\n'
  done < "$ORDER_FILE"

  # Add new sessions to order file
  echo "$all_sessions" | while IFS= read -r s; do
    [ -z "$s" ] && continue
    grep -qxF "$s" "$ORDER_FILE" 2>/dev/null || echo "$s" >> "$ORDER_FILE"
  done

  # Rebuild with any new sessions
  sessions=""
  while IFS= read -r s; do
    [ -z "$s" ] && continue
    echo "$all_sessions" | grep -qxF "$s" && sessions="$sessions$s"$'\n'
  done < "$ORDER_FILE"

  # Build display with indices, current marker, and hidden indicator
  i=1
  while IFS= read -r s; do
    [ -z "$s" ] && continue
    if grep -qxF "$s" "$HIDDEN_FILE"; then
      line="${YELLOW}${i}: ${s} 󰘓${RESET}"
    elif [ "$s" = "$current" ]; then
      line="${i}: ${s} ${GRAY}←${RESET}"
    else
      line="${i}: ${s}"
    fi
    echo -e "$line"
    i=$((i + 1))
  done <<< "$sessions"
}

if [ "$1" = "--popup" ]; then
  export -f build_list
  export ORDER_FILE HIDDEN_FILE SCRIPTS_DIR

  selected=$(build_list | fzf \
    --height=100% \
    --reverse \
    --ansi \
    --prompt=" Session: " \
    --header=$'\033[90malt-h:\033[0m hidden \033[90m│ alt-j/k:\033[0m move' \
    --header-first \
    --bind "alt-h:execute-silent($SCRIPTS_DIR/session-hide-toggle.sh {2})+reload(bash -c build_list)" \
    --bind "alt-j:execute-silent($SCRIPTS_DIR/session-move.sh down {2})+reload(bash -c build_list)+up" \
    --bind "alt-k:execute-silent($SCRIPTS_DIR/session-move.sh up {2})+reload(bash -c build_list)+down" \
    --color="bg+:#313244,fg+:#cdd6f4,hl:#f9e2af,hl+:#f9e2af,info:#89b4fa,prompt:#f9e2af,pointer:#f38ba8,marker:#a6e3a1,spinner:#f5c2e7,header:#6c7086")

  [ -z "$selected" ] && exit 0
  # Strip: index prefix, hidden icon, current marker, ANSI codes
  session=$(echo "$selected" | sed 's/\x1b\[[0-9;]*m//g; s/^[0-9]*: //; s/ 󰘓$//; s/ ←$//')
  [ -n "$session" ] && tmux switch-client -t "$session"
  exit 0
fi

# Launch popup
popup_bg=$(tmux show -gqv @popup_bg)
popup_border=$(tmux show -gqv @popup_border)
tmux display-popup -E -b rounded -w 80 -h 90% -x C -y C \
  -s "bg=$popup_bg" -S "fg=$popup_border,bg=$popup_bg" \
  "$0 --popup" || true
