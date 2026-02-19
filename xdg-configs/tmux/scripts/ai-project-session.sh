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

LOG_DIR="$HOME/.local/state/ai-session"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/$(date +%Y%m%d-%H%M%S).log"

# Gather worktree context for Claude
gather_worktree_context() {
  local context=""

  # Scan bare repos with worktrees
  for repo in ~/src/*.git; do
    [[ -d "$repo" ]] || continue
    local repo_name=$(basename "$repo" .git)
    context+="Repository: $repo_name ($repo)"$'\n'

    # List worktrees
    while IFS= read -r line; do
      local wt_path=$(echo "$line" | awk '{print $1}')
      local wt_name=$(basename "$wt_path")

      [[ "$line" == *"(bare)"* ]] && continue

      local branch=""
      if [[ "$line" == *"(detached HEAD)"* ]]; then
        branch="detached HEAD"
      else
        branch=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\].*/\1/p')
      fi

      context+="  Worktree: $wt_name @ $wt_path (branch: $branch)"$'\n'
    done < <(git -C "$repo" worktree list 2>/dev/null)

    # Include some remote branches for context
    local remote_branches=$(git -C "$repo" branch -r --list 'origin/*' 2>/dev/null | grep -v HEAD | sed 's|origin/||' | head -20 | tr '\n' ', ' || true)
    [[ -n "$remote_branches" ]] && context+="  Remote branches: $remote_branches"$'\n'
    context+=$'\n'
  done

  # Scan regular repos in ~/src (non-bare)
  for repo in ~/src/*/; do
    [[ -d "$repo/.git" ]] || continue
    local repo_name=$(basename "$repo")
    local branch=$(git -C "$repo" rev-parse --abbrev-ref HEAD 2>/dev/null)
    context+="Repository: $repo_name ($repo)"$'\n'
    context+="  Directory: $repo (branch: $branch)"$'\n'
    context+=$'\n'
  done

  echo "$context"
}


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

worktree_context=$(gather_worktree_context)

# Prompt for session name
session_name=$(gum input --prompt="Session name: " --placeholder="e.g., myfeature")

[[ -z "$session_name" ]] && exit 0

# Check if session already exists
if tmux has-session -t "$session_name" 2>/dev/null; then
  echo -e "${YELLOW}Session '$session_name' exists, switching...${RESET}"
  sleep 0.3
  tmux switch-client -t "$session_name"
  exit 0
fi

# Build system prompt for Claude with full context
system_prompt="You are an AI assistant embedded in a tmux session named '$session_name'.

USER REQUEST: $request

AVAILABLE WORKTREES AND REPOS:
$worktree_context

YOUR JOB:
Set up this tmux session for the user's request using tmux commands. Use these commands to configure the session:
- Create windows: tmux new-window -t '$session_name' -n <name> -c <dir>
- Send commands: tmux send-keys -t '$session_name:<window>' '<cmd>' Enter
- Select a window: tmux select-window -t '$session_name:<window>'
- Run git operations as needed (checkout, create branches, etc.)

A 'claude' window already exists (this one). Create additional windows as needed for the task (e.g., vi, sh, test).

When you are done setting up the session, tell the user what you did and that they can exit this window (type 'exit') to go to the main working window."

# Write system prompt to temp file so it is not expanded in send-keys
prompt_file="$LOG_DIR/system-prompt-$$.txt"
printf '%s' "$system_prompt" > "$prompt_file"

# Create new session with a claude window
echo -e "${GREEN}Creating session '$session_name'...${RESET}"
tmux new-session -d -s "$session_name" -n "claude" -c "$HOME"

# Start Claude with the system prompt (cat runs in the new session's shell, not here)
tmux send-keys -t "$session_name:claude" "claude --append-system-prompt \"\$(cat '$prompt_file')\"" Enter

sleep 0.2
tmux switch-client -t "$session_name"
