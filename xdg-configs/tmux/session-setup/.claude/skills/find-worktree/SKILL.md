---
description: "Safely discover available worktrees and repos for session setup"
---

# Find Worktree

Scan available git worktrees and repos under `~/src/` and report their status for session setup.

## Safety Rules (CRITICAL)

- NEVER `cd` into any directory
- NEVER run `git checkout`, `git reset`, `git clean`, `git branch -D`, or any write operation
- ALL git commands MUST use `git -C <path>` form
- Read-only operations only

## Scan Procedure

### 1. Bare repos (`~/src/*.git`)

For each directory matching `~/src/*.git`:

```bash
git -C ~/src/<name>.git worktree list --porcelain
```

Parse each worktree entry to extract:
- `worktree <path>` — the worktree path
- `branch refs/heads/<branch>` or `detached` — branch status
- `HEAD <sha>` — commit hash

For each worktree path found, check dirty status:

```bash
git -C <worktree-path> status --porcelain
```

### 2. Regular repos (`~/src/*/`)

For each directory matching `~/src/*/` that contains a `.git` entry (file or directory):

```bash
git -C ~/src/<name> worktree list --porcelain
```

Then check dirty status for each worktree path:

```bash
git -C <worktree-path> status --porcelain
```

## Status Classification

For each discovered worktree:

- **AVAILABLE** — `git status --porcelain` output is empty AND branch is not `main`/`master`
- **IN_USE** — `git status --porcelain` has output (count lines as uncommitted changes)
- **MAIN_BRANCH** — branch is `main` or `master` (avoid using these)

Detached HEAD worktrees are typically free/available — flag these as best candidates.

## Output Format

Produce structured output for each discovered worktree:

```
WORKTREE_SCAN_START
REPO: <repo-name>
  WORKTREE: <full-path>
    BRANCH: <branch-name> | DETACHED_HEAD:<sha>
    STATUS: AVAILABLE | IN_USE (<n> uncommitted changes) | MAIN_BRANCH
    CANDIDATE: YES | NO
REPO: ...
WORKTREE_SCAN_END

RECOMMENDED_DIR: <full-path-to-best-worktree>
RECOMMENDED_REPO: <repo-name>
ALTERNATIVES:
  - <path> (<branch>, <status>)
  - <path> (<branch>, <status>)
```

### Selection Priority

1. Detached HEAD + clean → highest priority candidate
2. Named branch + clean + not main/master → good candidate
3. Clean main/master worktree → last resort
4. Dirty worktree → not recommended (mark IN_USE)

If no suitable worktree exists, set `RECOMMENDED_DIR: NONE` and explain in a `NOTE:` line.
