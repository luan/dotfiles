#!/usr/bin/env bash
# AI-assisted tmux session creation
# Uses Claude to parse natural language requests and create appropriate sessions

set -eo pipefail

YELLOW="\033[33m"
GRAY="\033[90m"
CYAN="\033[36m"
GREEN="\033[32m"
RESET="\033[0m"

ICON_AI=$'\uf06a4  '  # robot icon

export GUM_INPUT_PROMPT_FOREGROUND="#f9e2af"
export GUM_INPUT_CURSOR_FOREGROUND="#f38ba8"

echo ""
echo -e "${CYAN}${ICON_AI}${RESET}AI-assisted session creation"
echo -e "${GRAY}Examples: 'work on branch luan/fix-auth', 'new branch for oauth work',${RESET}"
echo -e "${GRAY}'pr-superfresh on myproject.git wt1', 'repurpose wt3 for new feature'${RESET}"
echo ""

request=$(gum write --width=70 --height=4 --header="${ICON_AI}What do you want?" --placeholder="e.g., work on branch luan/fix-auth" --char-limit=500)

[[ -z "$request" ]] && exit 0

echo -e "${YELLOW}Request:${RESET} $request"
echo ""

# Fast-path: check if request exactly matches a directory or branch
fast_path_match() {
  local query="$1"
  local candidates=()

  # Check directories in priority order: ~/src, ~/.config, ~ (git repos first)
  for base in "$HOME/src" "$HOME/.config" "$HOME"; do
    for candidate in "$base/$query" "$base/.$query"; do
      if [[ -d "$candidate" ]]; then
        # Git repos get priority, non-git go to back
        if [[ -d "$candidate/.git" ]] || [[ -f "$candidate/HEAD" ]]; then
          echo "$candidate"
          return 0
        fi
        candidates+=("$candidate")
      fi
    done
  done

  # Return first non-git match if no git repo found
  if [[ ${#candidates[@]} -gt 0 ]]; then
    echo "${candidates[0]}"
    return 0
  fi

  # Check branch suffix match in worktrees
  for repo in ~/src/*.git; do
    [[ -d "$repo" ]] || continue
    while IFS= read -r line; do
      local wt_path=$(echo "$line" | awk '{print $1}')
      [[ "$line" == *"(bare)"* ]] && continue

      # Extract branch from [branch-name] using sed
      local branch=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\].*/\1/p')

      # Check if query matches branch suffix (after last /)
      if [[ -n "$branch" ]]; then
        local suffix="${branch##*/}"
        if [[ "$suffix" == "$query" ]]; then
          echo "$wt_path"
          return 0
        fi
      fi
    done < <(git -C "$repo" worktree list 2>/dev/null)
  done

  return 1
}

# Try fast-path first (single word, no spaces = likely directory or branch name)
if [[ ! "$request" =~ [[:space:]] ]]; then
  matched_dir=$(fast_path_match "$request" || true)
  if [[ -n "$matched_dir" ]]; then
    echo -e "${GREEN}Fast match:${RESET} $matched_dir"
    session_name="$request"
    final_dir="$matched_dir"

    # Check if session exists
    if tmux has-session -t "$session_name" 2>/dev/null; then
      echo -e "${YELLOW}Session '$session_name' exists, switching...${RESET}"
      sleep 0.3
      tmux switch-client -t "$session_name"
      exit 0
    fi

    # Create session directly
    echo -e "${GREEN}Creating session '$session_name' in $final_dir${RESET}"
    tmux new-session -d -s "$session_name" -n "ai" -c "$final_dir"
    tmux new-window -t "$session_name" -n "vi" -c "$final_dir"
    tmux new-window -t "$session_name" -n "sh" -c "$final_dir"
    tmux select-window -t "$session_name:ai"
    sleep 0.2
    tmux switch-client -t "$session_name"
    exit 0
  fi
fi

# Use a temporary session name; Claude will rename it to something meaningful
session_name="setup-$$"

# Sandbox dir: stowed path for the session-setup project
sandbox_dir="$HOME/.config/tmux/session-setup"

echo -e "${GREEN}Starting AI session setup...${RESET}"
tmux new-session -d -s "$session_name" -n "claude" -c "$sandbox_dir"

# Write request to temp file to avoid shell injection in send-keys
request_dir="$HOME/.local/state/ai-session"
mkdir -p "$request_dir"
# Clean stale request files from previous runs (older than 1 minute)
find "$request_dir" -name 'request-*.txt' -mmin +1 -delete 2>/dev/null || true
request_file="$request_dir/request-$$.txt"
printf '%s' "$request" > "$request_file"
(sleep 30 && rm -f "$request_file") &

# Start Claude in the sandbox dir with the user's request as the initial message.
# Claude picks up CLAUDE.md, .claude/settings.json, and skills from the sandbox.
tmux send-keys -t "$session_name:claude" "claude \"\$(cat '$request_file')\"" Enter

sleep 0.2
tmux switch-client -t "$session_name"
