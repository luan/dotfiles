#!/usr/bin/env bash
# Safely ditch a tmux session - detach HEAD and close
# Fails if there are uncommitted/unpushed changes or windows in different directories

set -eo pipefail

RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
GRAY="\033[90m"
RESET="\033[0m"

session=$(tmux display-message -p '#S')
echo -e "${YELLOW}Ditching session: $session${RESET}"
echo ""

# Get all pane directories in current session
dirs=$(tmux list-panes -s -t "$session" -F '#{pane_current_path}' | sort -u)
dir_count=$(echo "$dirs" | wc -l | tr -d ' ')

if [[ "$dir_count" -ne 1 ]]; then
  echo -e "${RED}✗ Windows are in different directories:${RESET}"
  echo "$dirs" | while read -r d; do echo -e "  ${GRAY}$d${RESET}"; done
  read -p "Press enter to exit..."
  exit 1
fi

dir=$(echo "$dirs" | head -1)
echo -e "${GREEN}✓${RESET} All windows in: ${GRAY}${dir/#$HOME/\~}${RESET}"

# Check if git repo
if ! git -C "$dir" rev-parse --git-dir >/dev/null 2>&1; then
  echo -e "${RED}✗ Not a git repository${RESET}"
  read -p "Press enter to exit..."
  exit 1
fi
echo -e "${GREEN}✓${RESET} Git repository"

# Check for uncommitted changes
if ! git -C "$dir" diff --quiet HEAD 2>/dev/null || \
   ! git -C "$dir" diff --cached --quiet 2>/dev/null; then
  echo -e "${RED}✗ Uncommitted changes:${RESET}"
  git -C "$dir" status --short
  read -p "Press enter to exit..."
  exit 1
fi

# Check for untracked files (optional, but good to warn)
untracked=$(git -C "$dir" ls-files --others --exclude-standard 2>/dev/null)
if [[ -n "$untracked" ]]; then
  echo -e "${YELLOW}! Untracked files (will be kept):${RESET}"
  echo "$untracked" | head -5 | while read -r f; do echo -e "  ${GRAY}$f${RESET}"; done
  [[ $(echo "$untracked" | wc -l) -gt 5 ]] && echo -e "  ${GRAY}...${RESET}"
fi
echo -e "${GREEN}✓${RESET} No uncommitted changes"

# Check for unpushed commits
branch=$(git -C "$dir" rev-parse --abbrev-ref HEAD 2>/dev/null)
if [[ "$branch" != "HEAD" ]]; then
  upstream=$(git -C "$dir" rev-parse --abbrev-ref '@{upstream}' 2>/dev/null || true)
  if [[ -n "$upstream" ]]; then
    unpushed=$(git -C "$dir" log --oneline "$upstream..HEAD" 2>/dev/null || true)
    if [[ -n "$unpushed" ]]; then
      echo -e "${RED}✗ Unpushed commits on $branch:${RESET}"
      echo "$unpushed" | while read -r c; do echo -e "  ${GRAY}$c${RESET}"; done
      read -p "Press enter to exit..."
      exit 1
    fi
  fi
fi
echo -e "${GREEN}✓${RESET} No unpushed changes"

echo ""

# Check if this is a worktree of a bare repo (has multiple worktrees)
is_worktree=false
git_common_dir=$(git -C "$dir" rev-parse --git-common-dir 2>/dev/null || true)
if [[ -n "$git_common_dir" && "$git_common_dir" != ".git" && "$git_common_dir" != "$dir/.git" ]]; then
  is_worktree=true
fi

if $is_worktree; then
  if ! gum confirm "Detach HEAD and kill session '$session'?"; then
    exit 0
  fi
  echo -e "${GREEN}Detaching HEAD...${RESET}"
  git -C "$dir" checkout --detach 2>/dev/null || true
else
  if ! gum confirm "Kill session '$session'?"; then
    exit 0
  fi
fi

# Kill session
echo -e "${GREEN}Killing session '$session'...${RESET}"
tmux kill-session -t "$session"
