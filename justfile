dotfiles_dir := justfile_directory()
config_dir := env("HOME") / ".config"

# List available recipes
default:
    @just --list

# Link stow configs + create convenience symlinks in dotfiles dir
link:
    stow -R xdg-configs -t "{{ config_dir }}"
    stow -R bin -t "{{ env("HOME") }}/bin"
    ln -sfn "{{ config_dir }}/nvim" "{{ dotfiles_dir }}/nvim"
    ln -sfn "$HOME/.claude" "{{ dotfiles_dir }}/dot-claude"

# Unlink stow configs + remove convenience symlinks
unlink:
    stow -D xdg-configs -t "{{ config_dir }}"
    stow -D bin -t "{{ env("HOME") }}/bin"
    rm -f "{{ dotfiles_dir }}/nvim"
    rm -f "{{ dotfiles_dir }}/dot-claude"

# Clone external repos if not already present
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

# Safely pull dotfiles + external repos (skips repos with uncommitted changes)
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

# Install Claude Code plugin marketplaces and plugins
claude-plugins:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v claude &>/dev/null; then
        echo "⚠ claude not found, skipping plugin setup"
        exit 0
    fi

    marketplaces=(
        "anthropics/claude-plugins-official"
        "steveyegge/beads"
    )

    plugins=(
        "beads@beads-marketplace"
        "clangd-lsp@claude-plugins-official"
        "claude-md-management@claude-plugins-official"
        "code-simplifier@claude-plugins-official"
        "context7@claude-plugins-official"
        "gopls-lsp@claude-plugins-official"
        "pyright-lsp@claude-plugins-official"
        "rust-analyzer-lsp@claude-plugins-official"
        "swift-lsp@claude-plugins-official"
    )

    for m in "${marketplaces[@]}"; do
        echo "→ Marketplace: $m"
        claude plugin marketplace add "$m" 2>/dev/null || true
    done

    for p in "${plugins[@]}"; do
        echo "→ Plugin: $p"
        claude plugin install "$p" 2>/dev/null || true
    done

    echo "✓ Claude plugins ready"

# Full setup: brew, repos, link, gitconfig, claude-plugins
setup: brew repos link gitconfig claude-plugins
