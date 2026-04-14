# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

macOS dotfiles managed with GNU Stow and orchestrated via justfile. Configs for fish, tmux, lazygit, ghostty, zed, plus custom automation scripts.

## Commands

```bash
just setup          # Full setup: brew + repos + link + gitconfig + claude-plugins
just link           # Stow configs to ~/.config + ~/bin, create convenience symlinks
just unlink         # Remove all stow symlinks
just repos          # Clone external repos (nvim, dot-claude) if missing
just pull           # Pull dotfiles + external repos (skips dirty repos)
just brew           # Install Homebrew packages from Brewfile
just macos-defaults # Apply macOS system preferences
just gitconfig      # Add gitconfig include to ~/.gitconfig
just claude-plugins # Install Claude Code plugin marketplaces and plugins
```

## Architecture

**Stow layout** — two stow directories, each targeting a different destination:
- `xdg-configs/` → stowed to `~/.config/` (fish, tmux, lazygit, ghostty, zed)
- `bin/` → stowed to `~/bin/` (custom scripts: ralph, bd-loop, loop-format.py)

**External repos** — independent git repos cloned separately, symlinked into dotfiles root for convenience. These are **not submodules** — they have their own git history and remotes. Commits to these repos must be made from within them.
- `./nvim` → `~/.config/nvim` (github.com/luan/nvim) — full Neovim config
- `./dot-claude` → `~/.claude` (github.com/luan/dot-claude) — Claude Code settings, CLAUDE.md, rules, hooks, skills, MCP config

See [docs/architecture.md](docs/architecture.md) for the layered diagram + module map.

**Root-level files** (not stowed):
- `gitconfig` / `themes.gitconfig` — sourced via `[include]` in `~/.gitconfig`
- `Brewfile` — Homebrew bundle
- `macos-defaults.sh` — macOS system preferences script

## Key Custom Scripts

- **ralph** (`bin/ralph`) — automated Claude Code execution loop. Reads prompts from `ralph/PROMPT_*.md`. Modes: `build`, `plan`, `spec`, `wait "cmd"`. Supports mode sequences with repeat counts and multi-model selection.
- **bd-loop** (`bin/bd-loop`) — automated Beads workflow loop using tmux. Modes: `build` (implement), `plan` (explore).
- **loop-format.py** (`bin/loop-format.py`) — output formatter for ralph/bd-loop with markdown rendering.

## Tmux Config

`xdg-configs/tmux/` has extensive session management scripts in `scripts/`. Plugins managed by tpm. The tmux config lives at `~/.config/tmux/tmux.conf` (XDG path, not `~/.tmux.conf`).

When changing `mux` (xdg-configs/tmux/mux/), run `just mux` to rebuild, copy, and codesign the runtime binary.

## Conventions

- Branch prefix: `luan/`
- Lazygit default branch prefix: `luan/`
- Fish shell is the primary shell (zsh/bash present but fish is configured)
- **Terminal emulator: WezTerm** (`xdg-configs/wezterm/wezterm.lua`). Ghostty config exists but is **not** in active use — edits to terminal-level keybindings (Cmd+*, CSI relays, sidebar wiring) go in the WezTerm config, not Ghostty.
- Catppuccin Mocha is the theme across tools (tmux, zed, lazygit)
- `.stow-local-ignore` skips `.DS_Store` files
