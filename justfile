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
    ln -sfn "{{ dotfiles_dir }}/xdg-configs/zsh/.zshenv" "{{ env("HOME") }}/.zshenv"

# Unlink stow configs + remove convenience symlinks
unlink:
    stow -D xdg-configs -t "{{ config_dir }}"
    stow -D bin -t "{{ env("HOME") }}/bin"
    rm -f "{{ dotfiles_dir }}/nvim"
    rm -f "{{ dotfiles_dir }}/dot-claude"
    rm -f "{{ env("HOME") }}/.zshenv"

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

# Resolve and lock sheldon plugins (called during setup)
sheldon:
    sheldon --config-file "{{ dotfiles_dir }}/xdg-configs/sheldon/plugins.toml" lock --update

# Set Homebrew zsh as login shell (registers in /etc/shells if missing; needs sudo for that step)
chsh-zsh:
    #!/usr/bin/env bash
    set -euo pipefail
    ZSH_BIN="/opt/homebrew/bin/zsh"

    if [ ! -x "$ZSH_BIN" ]; then
        echo "✗ $ZSH_BIN not found — run 'just brew' first" >&2
        exit 1
    fi

    if ! grep -qxF "$ZSH_BIN" /etc/shells; then
        echo "→ Registering $ZSH_BIN in /etc/shells (sudo)"
        echo "$ZSH_BIN" | sudo tee -a /etc/shells >/dev/null
    fi

    current=$(dscl . -read "/Users/$USER" UserShell 2>/dev/null | awk '{print $2}')
    if [ "$current" = "$ZSH_BIN" ]; then
        echo "✓ Login shell already $ZSH_BIN"
    else
        echo "→ Changing login shell: $current → $ZSH_BIN"
        chsh -s "$ZSH_BIN"
    fi

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
    )

    plugins=(
        "clangd-lsp@claude-plugins-official"
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

# Install cargo binaries via cargo-binstall
cargo:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v cargo-binstall &>/dev/null; then
        echo "⚠ cargo-binstall not found, run 'just brew' first"
        exit 1
    fi

    crates=(
        "ck-search"
    )

    for crate in "${crates[@]}"; do
        echo "→ $crate"
        cargo binstall "$crate" --no-confirm --quiet 2>/dev/null || echo "✗ $crate install failed"
    done

    echo "✓ Cargo binaries ready"

# Set up local dev-routing (Caddy + dnsmasq subdomain routing)
dev-routing: link
    #!/usr/bin/env bash
    set -euo pipefail
    "$HOME/bin/dev-routing" setup && "$HOME/bin/dev-routing" scan

# Run dot-claude setup (ct tool + completions)
dot-claude:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f "$HOME/.claude/justfile" ]; then
        echo "→ Running dot-claude setup"
        just -f "$HOME/.claude/justfile" setup
    else
        echo "⚠ dot-claude justfile not found, skipping"
    fi

# Build mux binary (Rust)
mux:
    cargo build --release --manifest-path="{{ dotfiles_dir }}/xdg-configs/tmux/mux/Cargo.toml"
    mkdir -p "{{ env("HOME") }}/bin" "{{ config_dir }}/tmux/scripts"
    cp "{{ dotfiles_dir }}/xdg-configs/tmux/mux/target/release/mux" "{{ env("HOME") }}/bin/mux"
    codesign --force --sign - "{{ env("HOME") }}/bin/mux"
    rm -f "{{ config_dir }}/tmux/scripts/mux"
    ln -s "{{ env("HOME") }}/bin/mux" "{{ config_dir }}/tmux/scripts/mux"
    swiftc -O -o "{{ env("HOME") }}/bin/notch-state" "{{ dotfiles_dir }}/xdg-configs/tmux/mux/scripts/notch-state.swift"
    codesign --force --sign - "{{ env("HOME") }}/bin/notch-state"
    @echo "✓ mux built"

# Full setup: brew, cargo, repos, link, gitconfig, claude-plugins, dev-routing, dot-claude, mux, sheldon
setup: brew cargo repos link gitconfig claude-plugins dev-routing dot-claude mux sheldon
