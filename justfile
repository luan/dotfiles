dotfiles_dir := justfile_directory()
config_dir := env("HOME") / ".config"

# List available recipes
default:
    @just --list

# Link all xdg-configs into ~/.config via stow
link:
    stow -R xdg-configs -t "{{ config_dir }}"
    stow -R bin -t "{{ env("HOME") }}/bin"

# Unlink all stow-managed configs
unlink:
    stow -D xdg-configs -t "{{ config_dir }}"
    stow -D bin -t "{{ env("HOME") }}/bin"

# Set up external repos (nvim, dot-claude) if not already cloned
repos:
    #!/usr/bin/env bash
    set -euo pipefail

    clone_if_missing() {
        local url="$1" dest="$2"
        if [ -d "$dest/.git" ]; then
            echo "✓ $dest already exists"
        else
            echo "→ Cloning $url into $dest"
            git clone "$url" "$dest"
        fi
    }

    clone_if_missing "https://github.com/luan/nvim" "{{ config_dir }}/nvim"
    clone_if_missing "https://github.com/luan/dot-claude" "$HOME/.claude"

# Safely pull dotfiles + all external repos (skips repos with uncommitted changes)
pull:
    #!/usr/bin/env bash
    set -euo pipefail

    safe_pull() {
        local dir="$1" name="$2"
        if [ ! -d "$dir/.git" ]; then
            echo "⚠ $name: not a git repo, skipping"
            return
        fi
        if [ -n "$(git -C "$dir" status --porcelain)" ]; then
            echo "⚠ $name: uncommitted changes, skipping"
            return
        fi
        echo "→ Pulling $name"
        git -C "$dir" pull --rebase --quiet && echo "✓ $name up to date" || echo "✗ $name pull failed"
    }

    safe_pull "{{ dotfiles_dir }}" "dotfiles"
    safe_pull "{{ config_dir }}/nvim" "nvim"
    safe_pull "$HOME/.claude" "dot-claude"

# Install Homebrew packages from Brewfile
brew:
    brew bundle --file="{{ dotfiles_dir }}/Brewfile"

# Apply macOS system defaults
macos-defaults:
    source "{{ dotfiles_dir }}/macos-defaults.sh"

# Set up git config include
gitconfig:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! grep -q "path={{ dotfiles_dir }}/gitconfig" "$HOME/.gitconfig" 2>/dev/null; then
        echo -e "\n[include]\n  path={{ dotfiles_dir }}/gitconfig" >> "$HOME/.gitconfig"
        echo "✓ Added gitconfig include"
    else
        echo "✓ gitconfig already configured"
    fi

# Migrate tmux from standalone repo to dotfiles-managed stow config
migrate-tmux:
    #!/usr/bin/env bash
    set -euo pipefail
    tmux_dir="{{ config_dir }}/tmux"

    if [ -L "$tmux_dir" ] && readlink "$tmux_dir" | grep -q dotfiles; then
        echo "✓ tmux already managed by dotfiles"
        exit 0
    fi

    if [ -d "$tmux_dir/.git" ]; then
        echo "→ Migrating tmux from standalone repo to dotfiles"
        # Check for uncommitted changes first
        if [ -n "$(git -C "$tmux_dir" status --porcelain)" ]; then
            echo "✗ tmux repo has uncommitted changes — commit or stash first"
            exit 1
        fi
        plugins_backup=""
        if [ -d "$tmux_dir/plugins" ]; then
            plugins_backup="$(mktemp -d)"
            echo "→ Backing up tmux plugins to $plugins_backup"
            cp -R "$tmux_dir/plugins" "$plugins_backup/"
        fi
        echo "→ Removing old tmux repo"
        rm -rf "$tmux_dir"
        echo "→ Linking tmux via stow"
        stow -R xdg-configs -t "{{ config_dir }}"
        if [ -n "$plugins_backup" ] && [ -d "$plugins_backup/plugins" ]; then
            echo "→ Restoring tmux plugins"
            cp -R "$plugins_backup/plugins" "$tmux_dir/"
            rm -rf "$plugins_backup"
        fi
        echo "✓ tmux migrated to dotfiles"
    elif [ -d "$tmux_dir" ]; then
        echo "→ tmux dir exists but is not a git repo — removing and linking"
        rm -rf "$tmux_dir"
        stow -R xdg-configs -t "{{ config_dir }}"
        echo "✓ tmux linked via stow"
    else
        echo "→ No existing tmux config — linking via stow"
        stow -R xdg-configs -t "{{ config_dir }}"
        echo "✓ tmux linked via stow"
    fi

# Full setup: brew, link, repos, gitconfig
setup: brew link repos gitconfig
