#!/usr/bin/env bash
# Create a new tmux session with 3 windows (ai, vi, sh) in a selected directory

set -e

CACHE_FILE="$HOME/.cache/tmux-project-dirs"
CACHE_TTL=300
FAVORITES_FILE="$HOME/.config/tmux/.session-favorites"
STATE_DIR="$HOME/.config/tmux"
SCRIPT_PATH="$0"

# Colors (tokyonight-ish)
YELLOW="\033[33m"
GRAY="\033[90m"
CYAN="\033[36m"
RESET="\033[0m"

# Icons (nerd font, with double space after)
ICON_FAV=$'\uf005  '
ICON_PROJECT=$'\uf07b  '
ICON_WORKTREE=$'\uf126  '
ICON_SESSION=$'\uf044  '

mkdir -p "$STATE_DIR" "$(dirname "$CACHE_FILE")"
touch "$FAVORITES_FILE"

# Handle special commands first
if [[ "$1" == "--toggle-favorite" ]]; then
  dir="${2/#\~/$HOME}"
  if grep -qxF "$dir" "$FAVORITES_FILE" 2>/dev/null; then
    grep -vxF "$dir" "$FAVORITES_FILE" > "$FAVORITES_FILE.tmp" && mv "$FAVORITES_FILE.tmp" "$FAVORITES_FILE"
  else
    echo "$dir" >> "$FAVORITES_FILE"
  fi
  exit 0
fi

if [[ "$1" == "--list" ]]; then
  filter="${2:-all}"

  # Favorites first (with  prefix) - always shown
  while IFS= read -r fav; do
    [[ -z "$fav" ]] && continue
    [[ -d "$fav" ]] || continue
    display="${fav/#$HOME/}"
    echo -e "${YELLOW}${ICON_FAV}${RESET}${GRAY}~${RESET}$display"
  done < "$FAVORITES_FILE"

  # Filtered directories based on tab
  case "$filter" in
    home)
      # Just ~ and ~/dotfiles
      for dir in "$HOME" "$HOME/dotfiles"; do
        [[ -d "$dir" ]] || continue
        grep -qxF "$dir" "$FAVORITES_FILE" 2>/dev/null && continue
        display="${dir/#$HOME/}"
        [[ -z "$display" ]] && display=""
        echo -e "   ${GRAY}~${RESET}$display"
      done
      ;;
    config)
      for d in "$HOME"/.config/*/; do
        [[ -d "$d" ]] || continue
        dir="${d%/}"
        grep -qxF "$dir" "$FAVORITES_FILE" 2>/dev/null && continue
        display="${dir/#$HOME/}"
        echo -e "   ${GRAY}~${RESET}$display"
      done | sort
      ;;
    src)
      for d in "$HOME"/src/*/; do
        [[ -d "$d" ]] || continue
        dir="${d%/}"
        grep -qxF "$dir" "$FAVORITES_FILE" 2>/dev/null && continue
        display="${dir/#$HOME/}"
        echo -e "   ${GRAY}~${RESET}$display"
      done | sort
      ;;
    *)
      # All directories
      {
        echo "$HOME"
        echo "$HOME/dotfiles"
        for d in "$HOME"/src/*/; do [[ -d "$d" ]] && echo "${d%/}"; done
        for d in "$HOME"/.config/*/; do [[ -d "$d" ]] && echo "${d%/}"; done
      } | sort -u | while IFS= read -r dir; do
        grep -qxF "$dir" "$FAVORITES_FILE" 2>/dev/null && continue
        display="${dir/#$HOME/}"
        echo -e "   ${GRAY}~${RESET}$display"
      done
      ;;
  esac
  exit 0
fi

# Function to get cached or fresh directory list
get_dirs() {
  if [[ -f "$CACHE_FILE" ]]; then
    cache_age=$(( $(date +%s) - $(stat -f %m "$CACHE_FILE" 2>/dev/null || echo 0) ))
    if [[ "$cache_age" -lt "$CACHE_TTL" ]]; then
      cat "$CACHE_FILE"
      return
    fi
  fi
  {
    echo "$HOME"
    echo "$HOME/dotfiles"
    for d in "$HOME"/src/*/; do [[ -d "$d" ]] && echo "${d%/}"; done
    for d in "$HOME"/.config/*/; do [[ -d "$d" ]] && echo "${d%/}"; done
  } | sort -u | tee "$CACHE_FILE"
}

# Function to get branch for a directory
get_branch() {
  git -C "$1" rev-parse --abbrev-ref HEAD 2>/dev/null || echo ""
}

# Build display list with favorites at top
build_display_list() {
  bash "$SCRIPT_PATH" --list
}

# Show directory picker with tabs
selected=$(build_display_list | SHELL=/bin/bash fzf \
  --prompt="${ICON_PROJECT}Project: " \
  --header=$'\033[90mctrl-f:\033[0m toggle \uf005 \033[90m│ 1:\033[0m ~ \033[90m│ 2:\033[0m ~/.config \033[90m│ 3:\033[0m ~/src \033[90m│ 0:\033[0m all' \
  --header-first \
  --height=100% \
  --reverse \
  --ansi \
  --color="bg+:#313244,fg+:#cdd6f4,hl:#f9e2af,hl+:#f9e2af,info:#89b4fa,prompt:#f9e2af,pointer:#f38ba8,marker:#a6e3a1,spinner:#f5c2e7,header:#6c7086" \
  --bind="ctrl-f:execute-silent(bash '$SCRIPT_PATH' --toggle-favorite {-1})+reload(bash '$SCRIPT_PATH' --list)" \
  --bind="1:reload(bash '$SCRIPT_PATH' --list home)" \
  --bind="2:reload(bash '$SCRIPT_PATH' --list config)" \
  --bind="3:reload(bash '$SCRIPT_PATH' --list src)" \
  --bind="0:reload(bash '$SCRIPT_PATH' --list all)")

[[ -z "$selected" ]] && exit 0

# Extract actual path
path_part=$(echo "$selected" | sed 's/.*~//' | sed 's/^\///')
selected_dir="$HOME/$path_part"
[[ "$path_part" == "" ]] && selected_dir="$HOME"

# Check if it's a bare repo or has multiple worktrees
final_dir="$selected_dir"
branch_name=""

is_bare=false
[[ "$selected_dir" == *.git ]] && [[ -d "$selected_dir" ]] && is_bare=true

if [[ -d "$selected_dir/.git" ]] || $is_bare; then
  if $is_bare; then
    worktrees=$(git -C "$selected_dir" worktree list 2>/dev/null | grep -v "(bare)" || true)
  else
    worktrees=$(git -C "$selected_dir" worktree list 2>/dev/null || true)
  fi

  worktree_count=$(echo "$worktrees" | grep -c . || echo 0)

  if $is_bare || [[ "$worktree_count" -gt 1 ]]; then
    formatted_worktrees=""

    while IFS= read -r wt; do
      [[ -z "$wt" ]] && continue
      wt_path=$(echo "$wt" | awk '{print $1}')
      wt_name=$(basename "$wt_path")
      wt_branch=$(get_branch "$wt_path")

      if [[ -n "$wt_branch" ]]; then
        formatted_worktrees+="$wt_name ${GRAY}← $wt_branch${RESET}"$'\n'
      else
        formatted_worktrees+="$wt_name"$'\n'
      fi
    done <<< "$worktrees"

    selected_wt=$(echo -e "$formatted_worktrees" | SHELL=/bin/bash fzf \
      --prompt="${ICON_WORKTREE}Worktree: " \
      --height=100% \
      --reverse \
      --ansi \
      --color="bg+:#313244,fg+:#cdd6f4,hl:#f9e2af,hl+:#f9e2af,info:#89b4fa,prompt:#f9e2af,pointer:#f38ba8,marker:#a6e3a1,spinner:#f5c2e7,header:#6c7086")

    if [[ -n "$selected_wt" ]]; then
      wt_name=$(echo "$selected_wt" | awk '{print $1}')
      while IFS= read -r wt; do
        [[ -z "$wt" ]] && continue
        wt_path=$(echo "$wt" | awk '{print $1}')
        if [[ "$(basename "$wt_path")" == "$wt_name" ]]; then
          final_dir="$wt_path"
          branch_name=$(get_branch "$wt_path")
          break
        fi
      done <<< "$worktrees"
    fi
  else
    branch_name=$(get_branch "$selected_dir")
  fi
else
  branch_name=$(get_branch "$selected_dir")
fi

# Determine default session name (prefer branch, then directory)
default_name=$(basename "$final_dir" | sed 's/\.git$//')
[[ -n "$branch_name" ]] && default_name="$branch_name"

# Gum colors to match tokyonight/catppuccin
export GUM_INPUT_PROMPT_FOREGROUND="#f9e2af"
export GUM_INPUT_CURSOR_FOREGROUND="#f38ba8"

echo ""
session_name=$(gum input --prompt="${ICON_SESSION}Session: " --value="$default_name" --width=50)

[[ -z "$session_name" ]] && session_name="$default_name"

# Check if session already exists
if tmux has-session -t "$session_name" 2>/dev/null; then
  gum style --foreground="#f9e2af" "Session '$session_name' exists, switching..."
  sleep 0.3
  tmux switch-client -t "$session_name"
  exit 0
fi

# Create session
gum style --foreground="#a6e3a1" "Creating '$session_name'..."
tmux new-session -d -s "$session_name" -n "ai" -c "$final_dir"
tmux new-window -t "$session_name" -n "vi" -c "$final_dir"
tmux new-window -t "$session_name" -n "sh" -c "$final_dir"

# Start claude and nvim by default
tmux send-keys -t "$session_name:ai" "claude" Enter
tmux send-keys -t "$session_name:vi" "nvim" Enter

tmux select-window -t "$session_name:ai"
sleep 0.2
tmux switch-client -t "$session_name"
