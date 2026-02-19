---
description: "Create tmux windows and configure the session layout"
---

# Setup Session

You are setting up the tmux session layout for a new project. Follow these steps precisely.

## Step 1: Rename the Session

Rename the temporary bootstrapper session to a concise, descriptive name (2-3 words) that reflects the TASK being worked on — not the project name.

```
tmux rename-session '<name>'
```

Examples: `fix-auth-bug`, `add-dark-mode`, `refactor-api`, `write-tests`

## Step 2: Create Windows

The bootstrapper window already exists as the first window. Now create the remaining windows.

### Default Layout (ai / vi / sh)

Unless the user requests otherwise, create these three windows in order:

```
tmux new-window -t '<name>' -n ai -c '<project_dir>'
tmux send-keys -t '<name>:ai' "claude '<request>'" Enter
tmux new-window -t '<name>' -n vi -c '<project_dir>'
tmux new-window -t '<name>' -n sh -c '<project_dir>'
tmux select-window -t '<name>:ai'
```

Replace `<project_dir>` with the project directory and `<request>` with the user's original task/request.

## Layout Flexibility

Honor explicit layout requests. Common variations:

- **"just give me a shell"** → only create an `sh` window, skip `ai` and `vi`
- **"no vi"** → skip the `vi` window; create `ai` and `sh` only
- **"start two claude sessions"** → create two `ai` windows (`ai` and `ai2`), each with `claude` running
- **"no claude"** → create `vi` and `sh`, skip `ai`
- **Custom names** → use requested window names

## Key Invariant

An `ai` window with `claude` running in the project directory **MUST always be created** unless the user explicitly says otherwise.

## tmux Command Reference

```
tmux rename-session '<name>'
tmux new-window -t '<session>' -n <window-name> -c <dir>
tmux send-keys -t '<session>:<window>' '<cmd>' Enter
tmux select-window -t '<session>:<window>'
```

## Step 3: Report and Close

Briefly tell the user what was set up (session name, windows, what's running).

Then close the bootstrapper window (the first window, named "claude") and leave the user on the ai window:

```
tmux select-window -t '<name>:ai'
tmux kill-window -t '<name>:claude'
```

IMPORTANT: Do NOT use `tmux kill-pane` — after `select-window`, the current pane is the ai window, not the bootstrapper. Use `kill-window -t '<name>:claude'` to target the bootstrapper window by name.
