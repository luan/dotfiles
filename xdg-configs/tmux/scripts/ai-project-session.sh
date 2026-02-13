#!/usr/bin/env bash
# AI-assisted tmux session creation
# Uses Claude to parse natural language requests and create appropriate sessions

set -eo pipefail

YELLOW="\033[33m"
GRAY="\033[90m"
CYAN="\033[36m"
GREEN="\033[32m"
RED="\033[31m"
RESET="\033[0m"

ICON_AI=$'\uf06a4  '  # robot icon
ICON_BRANCH=$'\uf126  '

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

# Get remote branches for a repo
get_remote_branches() {
  local repo="$1"
  git -C "$repo" branch -r --list 'origin/*' 2>/dev/null | sed 's/^[* ]*//' | head -50
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

# Log request and context
{
  echo "=== REQUEST ==="
  echo "$request"
  echo ""
  echo "=== CONTEXT ==="
  echo "$worktree_context"
} >> "$LOG_FILE"

# Check if request involves repurposing (needs careful handling -> use opus)
model="sonnet"
if echo "$request" | grep -qiE 'repurpose|reset|overwrite|reuse.*for|replace'; then
  model="opus"
  echo -e "${YELLOW}Using opus for careful worktree repurposing decision...${RESET}"
elif echo "$request" | grep -qiE 'new worktree|create worktree|new grove|add worktree'; then
  model="opus"
  echo -e "${YELLOW}Using opus for worktree creation decision...${RESET}"

fi
prompt='You are a git worktree assistant. Parse the user request and decide which worktree to use.

AVAILABLE WORKTREES AND REPOS:
'"$worktree_context"'

USER REQUEST: '"$request"'

Respond with ONLY valid JSON (no markdown, no explanation).

SCHEMA OPTION A — Simple (3 fixed windows: ai, vi, sh):
{
  "action": "use_existing" | "create_branch" | "checkout_branch" | "repurpose" | "create_worktree",
  "repo": "repo.git",
  "worktree_path": "/full/path/to/worktree",
  "branch": "existing-branch-name-or-null",
  "new_branch": "new-branch-name-or-null",
  "grove_name": "worktree-grove-name-or-null",
  "session_name": "one-short-word",
  "actions": {
    "pre": ["shell commands to run before session creation"],
    "ai": "claude prompt or null",
    "vi": "command for vi window or null",
    "sh": "command for sh window or null"
  },
  "explanation": "brief explanation of what will happen"
}

SCHEMA OPTION B — Custom windows (for multi-pane or multi-directory layouts):
{
  "action": "use_existing" | "create_branch" | "checkout_branch" | "repurpose" | "create_worktree",
  "repo": "repo.git",
  "worktree_path": "/full/path/to/worktree",
  "branch": "existing-branch-name-or-null",
  "new_branch": "new-branch-name-or-null",
  "grove_name": "worktree-grove-name-or-null",
  "session_name": "one-short-word",
  "actions": {
    "pre": ["shell commands to run before session creation"]
  },
  "windows": [
    {
      "name": "window-name",
      "layout": "even-horizontal | even-vertical | main-horizontal | main-vertical | tiled | null",
      "panes": [
        { "dir": "/working/directory", "cmd": "command to run or null" }
      ]
    }
  ],
  "focus_window": "window-name-to-focus-or-null",
  "explanation": "brief explanation of what will happen"
}

Use Schema B when:
- User asks for split panes, side-by-side, or tiled layouts
- User mentions multiple directories or repos in one session
- User describes specific window/pane arrangements

Use Schema A (default) when:
- Simple "work on branch X" requests
- No layout preferences mentioned

EXAMPLES for Schema B:

Example 1 — "split with project X left and project Y right":
{
  "action": "use_existing",
  "repo": "project.git",
  "worktree_path": "/home/user/src/project.git/wt1",
  "session_name": "split",
  "actions": { "pre": [] },
  "windows": [
    {
      "name": "code",
      "layout": "even-horizontal",
      "panes": [
        { "dir": "/home/user/src/projectX", "cmd": "nvim" },
        { "dir": "/home/user/src/projectY", "cmd": "nvim" }
      ]
    },
    { "name": "ai", "layout": null, "panes": [{ "dir": "/home/user/src/projectX", "cmd": null }] },
    { "name": "sh", "layout": null, "panes": [{ "dir": "/home/user/src/projectX", "cmd": null }] }
  ],
  "focus_window": "code",
  "explanation": "Side-by-side editors for projectX and projectY"
}

Example 2 — "three windows for coding, testing, and AI":
{
  "action": "use_existing",
  "repo": "myapp.git",
  "worktree_path": "/home/user/src/myapp",
  "session_name": "myapp",
  "actions": { "pre": [] },
  "windows": [
    { "name": "code", "layout": null, "panes": [{ "dir": "/home/user/src/myapp", "cmd": "nvim" }] },
    { "name": "test", "layout": null, "panes": [{ "dir": "/home/user/src/myapp", "cmd": "npm test -- --watch" }] },
    { "name": "ai", "layout": null, "panes": [{ "dir": "/home/user/src/myapp", "cmd": null }] }
  ],
  "focus_window": "code",
  "explanation": "Three windows: editor, test watcher, and AI assistant"
}

Example 3 — "tiled layout watching 4 log files":
{
  "action": "use_existing",
  "repo": "infra.git",
  "worktree_path": "/home/user/src/infra",
  "session_name": "logs",
  "actions": { "pre": [] },
  "windows": [
    {
      "name": "logs",
      "layout": "tiled",
      "panes": [
        { "dir": "/var/log", "cmd": "tail -f /var/log/app1.log" },
        { "dir": "/var/log", "cmd": "tail -f /var/log/app2.log" },
        { "dir": "/var/log", "cmd": "tail -f /var/log/app3.log" },
        { "dir": "/var/log", "cmd": "tail -f /var/log/app4.log" }
      ]
    }
  ],
  "focus_window": "logs",
  "explanation": "Tiled 4-pane layout watching log files"
}

RULES:
1. WORKTREE SELECTION:
   - If user mentions a branch, find worktree already on that branch (use_existing)
   - If no worktree has the branch, find one at "detached HEAD" to checkout the branch (checkout_branch)
   - For new branches, pick a detached HEAD worktree (create_branch)
   - When user references multiple directories, use action "use_existing" with Schema B

2. BRANCH NAMING:
   - New branches MUST have "luan/" prefix (e.g., "auth feature" -> "luan/auth")
   - session_name: short word from branch (e.g., "luan/fix-auth" -> "auth")

3. ACTIONS:
   - pre: shell commands to run in worktree BEFORE session (e.g., ["gt sync", "npm install"])
   - ai: prompt for Claude Code - runs as `claude "<prompt>"` (e.g., "/pr-superfresh", "fix the bug")
   - vi: command for vi window (e.g., "nvim", "nvim src/auth.ts")
   - sh: command for sh window (e.g., "npm run dev", "cargo watch")
   - Set any to null if not needed

4. REPURPOSE: Only if user explicitly says repurpose/reset. This is destructive.

5. CREATE_WORKTREE: Use when no existing worktree is suitable AND no detached HEAD
   worktrees are available. Set grove_name to a short descriptive name for the new
   worktree (e.g., "auth", "refactor"). The repo field must be the bare repo name
   (e.g., "arc.git"). worktree_path is ignored for this action (set to "/tmp").
   new_branch is the branch to create in the new worktree (must have "luan/" prefix).

6. Handle ALL parts of the request - worktree selection AND any requested actions.

7. VALID LAYOUTS: even-horizontal, even-vertical, main-horizontal, main-vertical, tiled.
   Use null for single-pane windows.'

echo -e "${GRAY}Asking Claude ($model)...${RESET}"
echo "=== MODEL: $model ===" >> "$LOG_FILE"

response=$(claude --model "$model" -p "$prompt" 2>&1)

# Log response
{
  echo "=== RESPONSE ==="
  echo "$response"
  echo ""
} >> "$LOG_FILE"

# Try to parse JSON from response (handle markdown fences and multi-line)
json=$(echo "$response" | sed -n '/^{/,/^}/p' | tr -d '\n' || true)
if [[ -z "$json" ]]; then
  # Try extracting from markdown code block
  json=$(echo "$response" | sed -n '/```/,/```/p' | grep -v '```' | tr -d '\n' || true)
fi
# Last resort: extract anything between { and }
if [[ -z "$json" ]]; then
  json=$(echo "$response" | tr -d '\n' | sed 's/.*\({.*}\).*/\1/' || true)
fi

if [[ -z "$json" ]] || ! echo "$json" | jq . >/dev/null 2>&1; then
  echo -e "${RED}Failed to parse Claude response${RESET}"
  echo "$response"
  echo -e "${GRAY}Log: $LOG_FILE${RESET}"
  read -r -p "Press enter to exit..."
  exit 1
fi

# Extract fields using jq
action=$(echo "$json" | jq -r '.action // empty')
repo=$(echo "$json" | jq -r '.repo // empty')
worktree_path=$(echo "$json" | jq -r '.worktree_path // empty')
branch=$(echo "$json" | jq -r '.branch // empty')
grove_name=$(echo "$json" | jq -r '.grove_name // empty')
new_branch=$(echo "$json" | jq -r '.new_branch // empty')
session_name=$(echo "$json" | jq -r '.session_name // empty')
explanation=$(echo "$json" | jq -r '.explanation // empty')

# Extract actions
actions_pre=$(echo "$json" | jq -r '.actions.pre // [] | .[]' 2>/dev/null || true)
actions_ai=$(echo "$json" | jq -r '.actions.ai // empty' 2>/dev/null || true)
actions_vi=$(echo "$json" | jq -r '.actions.vi // empty' 2>/dev/null || true)
actions_sh=$(echo "$json" | jq -r '.actions.sh // empty' 2>/dev/null || true)

echo ""
echo -e "${CYAN}Plan:${RESET} $explanation"
echo -e "${GRAY}Action: $action | Worktree: $worktree_path | Session: $session_name${RESET}"

if [[ -n "$new_branch" && "$new_branch" != "null" ]]; then
  echo -e "${GRAY}New branch: $new_branch${RESET}"
fi
if [[ -n "$grove_name" && "$grove_name" != "null" ]]; then
  echo -e "${GRAY}Grove name: $grove_name${RESET}"
fi
if [[ -n "$actions_pre" ]]; then
  echo -e "${GRAY}Pre-commands: $(echo "$actions_pre" | tr '\n' '; ')${RESET}"
fi

# Display windows layout or legacy actions
has_windows_display=$(echo "$json" | jq 'if .windows and (.windows | length) > 0 then "yes" else "no" end' -r)
if [[ "$has_windows_display" == "yes" ]]; then
  echo -e "${CYAN}Windows:${RESET}"
  local_win_count=$(echo "$json" | jq '.windows | length')
  for ((i = 0; i < local_win_count; i++)); do
    local_wname=$(echo "$json" | jq -r ".windows[$i].name")
    local_layout=$(echo "$json" | jq -r ".windows[$i].layout // \"default\"")
    local_pane_count=$(echo "$json" | jq ".windows[$i].panes | length")
    echo -e "  ${GREEN}$local_wname${RESET} ($local_pane_count pane(s), layout: $local_layout)"
    for ((j = 0; j < local_pane_count; j++)); do
      local_pdir=$(echo "$json" | jq -r ".windows[$i].panes[$j].dir // \"(default)\"")
      local_pcmd=$(echo "$json" | jq -r ".windows[$i].panes[$j].cmd // \"(shell)\"")
      echo -e "    ${GRAY}pane $j: $local_pdir — $local_pcmd${RESET}"
    done
  done
  local_focus=$(echo "$json" | jq -r '.focus_window // "(first)"')
  echo -e "  ${GRAY}Focus: $local_focus${RESET}"
else
  if [[ -n "$actions_ai" && "$actions_ai" != "null" ]]; then
    echo -e "${GRAY}AI window: claude \"$actions_ai\"${RESET}"
  fi
  if [[ -n "$actions_vi" && "$actions_vi" != "null" ]]; then
    echo -e "${GRAY}VI window: $actions_vi${RESET}"
  fi
  if [[ -n "$actions_sh" && "$actions_sh" != "null" ]]; then
    echo -e "${GRAY}SH window: $actions_sh${RESET}"
  fi
fi

echo -e "${GRAY}Log: $LOG_FILE${RESET}"
echo ""
if ! gum confirm "Proceed?"; then
  exit 0
fi

final_dir="$worktree_path"

# Handle branch operations
case "$action" in
  create_branch)
    if [[ -n "$new_branch" && "$new_branch" != "null" ]]; then
      echo -e "${GREEN}Creating new branch: $new_branch${RESET}"
      cd "$worktree_path"
      git checkout main 2>/dev/null || git checkout master 2>/dev/null || true
      gt sync 2>/dev/null || git pull --rebase origin main 2>/dev/null || true
      gt create "$new_branch" 2>/dev/null || git checkout -b "$new_branch"
    fi
    ;;
  checkout_branch)
    if [[ -n "$branch" && "$branch" != "null" ]]; then
      echo -e "${GREEN}Checking out branch: $branch${RESET}"
      cd "$worktree_path"
      git checkout "$branch" 2>/dev/null || git checkout -b "$branch" "origin/$branch" 2>/dev/null || true
    fi
    ;;
  repurpose)
    if [[ -n "$branch" && "$branch" != "null" ]]; then
      echo -e "${YELLOW}Repurposing worktree for: $branch${RESET}"
      cd "$worktree_path"
      git checkout main 2>/dev/null || git checkout master 2>/dev/null || true
      git reset --hard HEAD
      git clean -fd
      gt sync 2>/dev/null || git pull --rebase origin main 2>/dev/null || true
      if [[ -n "$new_branch" && "$new_branch" != "null" ]]; then
        gt create "$new_branch" 2>/dev/null || git checkout -b "$new_branch"
      else
        git checkout "$branch" 2>/dev/null || git checkout -b "$branch" "origin/$branch" 2>/dev/null || true
      fi
    fi
    ;;
  create_worktree)
    repo_path="$HOME/src/$repo"
    if [[ ! -d "$repo_path" ]]; then
      echo -e "${RED}Repository not found: $repo_path${RESET}"
      read -r -p "Press enter to exit..."
      exit 1
    fi
    echo -e "${GREEN}Creating new worktree in $repo_path${RESET}"
    create_args=("--repo" "$repo_path")
    if [[ -n "$grove_name" && "$grove_name" != "null" ]]; then
      create_args+=("--name" "$grove_name")
    fi
    if [[ -n "$new_branch" && "$new_branch" != "null" ]]; then
      create_args+=("--branch" "$new_branch")
    fi
    if ! created_path=$("$HOME/bin/gg-create-worktree" "${create_args[@]}" 2>&1); then
      echo -e "${RED}Failed to create worktree: $created_path${RESET}"
      read -r -p "Press enter to exit..."
      exit 1
    fi
    if [[ -z "$created_path" ]]; then
      echo -e "${RED}gg-create-worktree returned empty path${RESET}"
      read -r -p "Press enter to exit..."
      exit 1
    fi
    final_dir="$created_path"
    echo -e "${GREEN}Worktree created: $final_dir${RESET}"
    ;;
esac

# Run pre-commands in the worktree
if [[ -n "$actions_pre" ]]; then
  echo -e "${GREEN}Running pre-commands...${RESET}"
  cd "$final_dir"
  while IFS= read -r cmd; do
    [[ -z "$cmd" ]] && continue
    echo -e "${GRAY}  > $cmd${RESET}"
    eval "$cmd" || echo -e "${YELLOW}  Warning: command failed${RESET}"
  done <<< "$actions_pre"
fi

# Default session name if empty
[[ -z "$session_name" || "$session_name" == "null" ]] && session_name=$(basename "$worktree_path")

# Check if session exists
if tmux has-session -t "$session_name" 2>/dev/null; then
  echo -e "${YELLOW}Session '$session_name' exists, switching...${RESET}"
  sleep 0.3
  tmux switch-client -t "$session_name"
  exit 0
fi

# Create session from windows array
create_session_from_windows() {
  local session="$1"
  local json="$2"
  local default_dir="$3"

  local valid_layouts="even-horizontal even-vertical main-horizontal main-vertical tiled"
  local win_count
  win_count=$(echo "$json" | jq '.windows | length')
  local focus_window
  focus_window=$(echo "$json" | jq -r '.focus_window // empty')

  local first_window_name=""

  for ((w = 0; w < win_count; w++)); do
    local wname wdir layout pane_count
    wname=$(echo "$json" | jq -r ".windows[$w].name")
    wdir=$(echo "$json" | jq -r ".windows[$w].panes[0].dir // empty")
    [[ -z "$wdir" || "$wdir" == "null" ]] && wdir="$default_dir"
    layout=$(echo "$json" | jq -r ".windows[$w].layout // empty")
    pane_count=$(echo "$json" | jq ".windows[$w].panes | length")

    # Track first window name for fallback focus
    [[ $w -eq 0 ]] && first_window_name="$wname"

    # Create window
    if [[ $w -eq 0 ]]; then
      tmux new-session -d -s "$session" -n "$wname" -c "$wdir"
    else
      tmux new-window -t "$session" -n "$wname" -c "$wdir"
    fi

    # Create additional panes (pane 0 already exists from window creation)
    for ((p = 1; p < pane_count; p++)); do
      local pdir
      pdir=$(echo "$json" | jq -r ".windows[$w].panes[$p].dir // empty")
      [[ -z "$pdir" || "$pdir" == "null" ]] && pdir="$default_dir"
      tmux split-window -t "$session:$wname" -c "$pdir"
    done

    # Apply layout if multi-pane and layout specified
    if [[ $pane_count -gt 1 && -n "$layout" && "$layout" != "null" ]]; then
      if echo "$valid_layouts" | grep -qw "$layout"; then
        tmux select-layout -t "$session:$wname" "$layout"
      else
        echo -e "${YELLOW}Unknown layout '$layout', skipping${RESET}"
      fi
    fi

    # Send commands to each pane
    for ((p = 0; p < pane_count; p++)); do
      local cmd
      cmd=$(echo "$json" | jq -r ".windows[$w].panes[$p].cmd // empty")
      if [[ -n "$cmd" && "$cmd" != "null" ]]; then
        tmux send-keys -t "$session:$wname.$p" "$cmd" Enter
      fi
    done
  done

  # Focus the requested window (or first window)
  local target="${focus_window:-$first_window_name}"
  tmux select-window -t "$session:$target"
}

# Create session — choose windows-array or legacy 3-window path
has_windows=$(echo "$json" | jq 'if .windows and (.windows | length) > 0 then "yes" else "no" end' -r)

if [[ "$has_windows" == "yes" ]]; then
  echo -e "${GREEN}Creating session '$session_name' with custom windows...${RESET}"
  create_session_from_windows "$session_name" "$json" "$final_dir"
else
  echo -e "${GREEN}Creating session '$session_name' in $final_dir${RESET}"
  tmux new-session -d -s "$session_name" -n "ai" -c "$final_dir"
  tmux new-window -t "$session_name" -n "vi" -c "$final_dir"
  tmux new-window -t "$session_name" -n "sh" -c "$final_dir"

  if [[ -n "$actions_vi" && "$actions_vi" != "null" ]]; then
    echo -e "${GREEN}VI window: $actions_vi${RESET}"
    tmux send-keys -t "$session_name:vi" "$actions_vi" Enter
  fi

  if [[ -n "$actions_sh" && "$actions_sh" != "null" ]]; then
    echo -e "${GREEN}SH window: $actions_sh${RESET}"
    tmux send-keys -t "$session_name:sh" "$actions_sh" Enter
  fi

  if [[ -n "$actions_ai" && "$actions_ai" != "null" ]]; then
    echo -e "${GREEN}AI window: claude \"$actions_ai\"${RESET}"
    tmux send-keys -t "$session_name:ai" "claude \"$actions_ai\"" Enter
  fi

  tmux select-window -t "$session_name:ai"
fi

sleep 0.2
tmux switch-client -t "$session_name"
