set windows-shell := ["pwsh", "-NoProfile", "-Command"]

dotfiles_dir := justfile_directory()
home := home_directory()
config_dir := home / ".config"

# List available recipes
default:
    @just --list

# Link stow configs + create convenience symlinks in dotfiles dir
[unix]
link:
    mkdir -p "{{ home }}/bin"
    mkdir -p "{{ config_dir }}"
    stow -R xdg-configs -t "{{ config_dir }}"
    stow -R bin -t "{{ home }}/bin"
    ln -sfn "{{ config_dir }}/nvim" "{{ dotfiles_dir }}/nvim"
    if [ ! -e "{{ home }}/.zshenv" ]; then printf '%s\n' '[[ -r "$HOME/.config/zsh/zshenv" ]] && source "$HOME/.config/zsh/zshenv"' > "{{ home }}/.zshenv"; fi
    if [ ! -e "{{ home }}/.zshrc" ]; then printf '%s\n' '_zsh_config_home="${ZSH_CONFIG_HOME:-$HOME/.config/zsh}"' '[[ -r "$_zsh_config_home/zshrc" ]] && source "$_zsh_config_home/zshrc"' 'unset _zsh_config_home' > "{{ home }}/.zshrc"; fi
    touch "{{ home }}/.bashrc"
    grep -qxF '[[ -r "$HOME/.config/bash/bashrc" ]] && source "$HOME/.config/bash/bashrc"' "{{ home }}/.bashrc" || printf '\n%s\n' '[[ -r "$HOME/.config/bash/bashrc" ]] && source "$HOME/.config/bash/bashrc"' >> "{{ home }}/.bashrc"
    if [ ! -e "{{ home }}/.bash_profile" ] && [ ! -e "{{ home }}/.bash_login" ] && [ ! -e "{{ home }}/.profile" ]; then printf '%s\n' '[[ -r "$HOME/.bashrc" ]] && source "$HOME/.bashrc"' > "{{ home }}/.bash_profile"; fi
    if [ -e "{{ home }}/.bash_profile" ]; then grep -qxF '[[ -r "$HOME/.bashrc" ]] && source "$HOME/.bashrc"' "{{ home }}/.bash_profile" || printf '\n%s\n' '[[ -r "$HOME/.bashrc" ]] && source "$HOME/.bashrc"' >> "{{ home }}/.bash_profile"; fi

# Symlink xdg-configs + bin into place (requires Developer Mode for symlinks)
[windows]
link: gitconfig
    #!pwsh
    $ErrorActionPreference = 'Stop'
    New-Item -ItemType Directory -Force '{{ home }}\bin' | Out-Null
    New-Item -ItemType Directory -Force '{{ config_dir }}' | Out-Null
    $links = @()
    foreach ($item in Get-ChildItem -Directory '{{ dotfiles_dir }}\xdg-configs') {
        $links += @{ Path = Join-Path '{{ config_dir }}' $item.Name; Target = $item.FullName }
    }
    foreach ($item in Get-ChildItem -File '{{ dotfiles_dir }}\bin') {
        $links += @{ Path = Join-Path '{{ home }}\bin' $item.Name; Target = $item.FullName }
    }
    foreach ($l in $links) {
        if (Test-Path $l.Path) {
            $existing = Get-Item $l.Path -Force
            if (-not $existing.LinkType) { Write-Output "⚠ $($l.Path) exists and is not a link, skipping"; continue }
            $existing.Delete()
        }
        New-Item -ItemType SymbolicLink -Path $l.Path -Target $l.Target | Out-Null
        Write-Output "✓ $($l.Path) → $($l.Target)"
    }
    if (-not [Environment]::GetEnvironmentVariable('XDG_CONFIG_HOME', 'User')) {
        [Environment]::SetEnvironmentVariable('XDG_CONFIG_HOME', (Join-Path $HOME '.config'), 'User')
        Write-Output "✓ set XDG_CONFIG_HOME user env var"
    }
    # $PROFILE may live on a shared/redirected Documents folder where symlinks
    # are unreliable, so bootstrap with a dot-sourcing stub instead (mirrors .zshenv).
    $stub = $PROFILE.CurrentUserAllHosts
    if (-not (Test-Path $stub)) {
        New-Item -ItemType Directory -Force (Split-Path $stub) | Out-Null
        Set-Content $stub '. (Join-Path $HOME ''.config\powershell\profile.ps1'')'
        Write-Output "✓ created $stub"
    }

# Unlink stow configs + remove convenience symlinks
[unix]
unlink:
    stow -D xdg-configs -t "{{ config_dir }}"
    stow -D bin -t "{{ home }}/bin"
    rm -f "{{ dotfiles_dir }}/nvim"

# Remove symlinks pointing into this repo
[windows]
unlink:
    #!pwsh
    $ErrorActionPreference = 'Stop'
    $roots = @(
        @{ Dir = '{{ config_dir }}'; Source = '{{ dotfiles_dir }}\xdg-configs' },
        @{ Dir = '{{ home }}\bin'; Source = '{{ dotfiles_dir }}\bin' }
    )
    foreach ($r in $roots) {
        if (-not (Test-Path $r.Dir)) { continue }
        foreach ($item in Get-ChildItem $r.Dir -Force) {
            if ($item.LinkType -and $item.LinkTarget -like "$($r.Source)*") {
                $item.Delete()
                Write-Output "✗ removed $($item.FullName)"
            }
        }
    }

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

# Install Homebrew-managed formulae, casks, services, and Mac App Store apps from Brewfile
brew:
    #!/usr/bin/env bash
    set -euo pipefail

    real_brew=""
    for candidate in /opt/homebrew/bin/brew /usr/local/bin/brew; do
        if [ -x "$candidate" ]; then
            real_brew="$candidate"
            break
        fi
    done
    if [ -z "$real_brew" ]; then
        echo "✗ Homebrew not found at /opt/homebrew/bin/brew or /usr/local/bin/brew" >&2
        exit 1
    fi

    "$real_brew" trust --tap moltenbits/tap >/dev/null 2>&1 || true
    "$real_brew" trust --formula oven-sh/bun/bun >/dev/null 2>&1 || true

    "$real_brew" bundle --file="{{ dotfiles_dir }}/Brewfile"

# Install Windows CLI tools via winget + PSFzf module
[windows]
winget:
    #!pwsh
    $pkgs = @(
        'Starship.Starship'
        'ajeetdsouza.zoxide'
        'junegunn.fzf'
        'eza-community.eza'
        'sharkdp.bat'
        'sharkdp.fd'
        'BurntSushi.ripgrep.MSVC'
        'dandavison.delta'
        'rsteube.Carapace'
        'JesseDuffield.lazygit'
        'jj-vcs.jj'
    )
    foreach ($p in $pkgs) {
        Write-Output "→ $p"
        winget install --id $p --exact --silent --accept-package-agreements --accept-source-agreements --disable-interactivity | Select-Object -Last 1
    }
    if (-not (Get-Module PSFzf -ListAvailable)) {
        Write-Output "→ PSFzf module"
        Install-Module PSFzf -Scope CurrentUser -Force
    }
    Write-Output "✓ Windows CLI tools ready"

# Resolve and lock sheldon plugins (called during setup)
sheldon:
    PATH="$HOME/.local/bin:/opt/homebrew/bin:/opt/homebrew/sbin:$PATH" sheldon --config-file "{{ dotfiles_dir }}/xdg-configs/sheldon/plugins.toml" lock --update

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
[unix]
gitconfig:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! grep -q "path={{ dotfiles_dir }}/gitconfig" "$HOME/.gitconfig" 2>/dev/null; then
        echo -e "\n[include]\n  path={{ dotfiles_dir }}/gitconfig" >> "$HOME/.gitconfig"
        echo "✓ Added gitconfig include"
    else
        echo "✓ gitconfig already configured"
    fi

# Set up git config include
[windows]
gitconfig:
    #!pwsh
    $ErrorActionPreference = 'Stop'
    $include = '{{ dotfiles_dir }}/gitconfig' -replace '\\', '/'
    $existing = git config --global --get-all include.path
    if ($existing -contains $include) {
        Write-Output "✓ gitconfig already configured"
    } else {
        git config --global --add include.path $include
        Write-Output "✓ Added gitconfig include"
    }

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
    export PATH="$HOME/.local/bin:/opt/homebrew/bin:/opt/homebrew/sbin:$PATH"

    if ! command -v cargo-binstall &>/dev/null; then
        echo "⚠ cargo-binstall not found, run 'just brew' first"
        exit 1
    fi

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

# Build mux binary (Rust)
mux:
    cargo build --release --manifest-path="{{ dotfiles_dir }}/xdg-configs/tmux/mux/Cargo.toml"
    mkdir -p "{{ home }}/bin" "{{ config_dir }}/tmux/scripts"
    cp "{{ dotfiles_dir }}/xdg-configs/tmux/mux/target/release/mux" "{{ home }}/bin/mux"
    codesign --force --sign - "{{ home }}/bin/mux"
    rm -f "{{ config_dir }}/tmux/scripts/mux"
    ln -s "{{ home }}/bin/mux" "{{ config_dir }}/tmux/scripts/mux"
    swiftc -O -o "{{ home }}/bin/notch-state" "{{ dotfiles_dir }}/xdg-configs/tmux/mux/scripts/notch-state.swift"
    codesign --force --sign - "{{ home }}/bin/notch-state"
    @echo "✓ mux built"

# Run mux sidebar performance regression guardrails
mux-perf:
    #!/usr/bin/env bash
    set -euo pipefail
    manifest="{{ dotfiles_dir }}/xdg-configs/tmux/mux/Cargo.toml"
    cargo test --manifest-path="$manifest"
    cargo bench --manifest-path="$manifest" --bench sidebar -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2
    alloc_output="$(mktemp)"
    cargo bench --manifest-path="$manifest" --bench sidebar_alloc -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2 2>&1 | tee "$alloc_output"
    python3 "{{ dotfiles_dir }}/xdg-configs/tmux/mux/scripts/check-sidebar-alloc-thresholds.py" "$alloc_output"
    rm -f "$alloc_output"
    profile_json="$(mktemp)"
    cargo run --manifest-path="$manifest" -- sidebar profile 4000 > "$profile_json"
    python3 -c 'import json, sys; p=json.load(open(sys.argv[1])); r={row["state"]: row["counters"] for row in p["sidebar"]}; assert r["visible-idle"]["redraws"] <= 1, r["visible-idle"]; assert r["visible-idle"]["tmux_spawns"] == 0, r["visible-idle"]; assert r["hidden-idle"]["redraws"] == 0, r["hidden-idle"]; assert r["hidden-idle"]["animation_frames"] == 0, r["hidden-idle"]; assert r["hidden-idle"]["tmux_spawns"] <= 1, r["hidden-idle"]; assert r["active-animation"]["animation_frames"] <= 122, r["active-animation"]; assert r["active-animation"]["tmux_spawns"] == 0, r["active-animation"]; assert p["daemon"]["meta_refresh_interval_ms"] >= 5000, p["daemon"]' "$profile_json"
    rm -f "$profile_json"
    echo "✓ mux sidebar perf guardrails passed"

# Run allocation-aware mux sidebar benchmark report
mux-memory:
    #!/usr/bin/env bash
    set -euo pipefail
    manifest="{{ dotfiles_dir }}/xdg-configs/tmux/mux/Cargo.toml"
    cargo bench --manifest-path="$manifest" --bench sidebar_alloc -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.2

# Profile live mux sidebar daemon memory in isolated tmux shapes
mux-memory-live:
    "{{ dotfiles_dir }}/xdg-configs/tmux/mux/scripts/sidebar-memory-profile.sh"

# Measure sidebar status propagation latency in an isolated tmux server
mux-status-latency:
    #!/usr/bin/env bash
    set -euo pipefail
    manifest="{{ dotfiles_dir }}/xdg-configs/tmux/mux/Cargo.toml"
    bin="{{ dotfiles_dir }}/xdg-configs/tmux/mux/target/release/mux"
    cargo build --release --manifest-path="$manifest" >/dev/null
    socket="mux-sidebar-status-latency-$$"
    tmp_home="$(mktemp -d)"
    wrapper_dir="$(mktemp -d)"
    tmux_bin="$(command -v tmux)"
    cleanup() {
        "$tmux_bin" -L "$socket" kill-server >/dev/null 2>&1 || true
        rm -rf "$tmp_home" "$wrapper_dir"
    }
    trap cleanup EXIT
    printf '%s\n' '#!/usr/bin/env bash' "exec \"$tmux_bin\" -L \"$socket\" \"\$@\"" > "$wrapper_dir/tmux"
    chmod +x "$wrapper_dir/tmux"
    "$tmux_bin" -L "$socket" new-session -d -x 120 -y 40 -s mux-latency "sleep 120"
    PATH="$wrapper_dir:$PATH" HOME="$tmp_home" "$bin" sidebar status-latency-profile 8 750

# Full setup: Homebrew, cargo, repos, link, gitconfig, claude-plugins, dev-routing, mux, sheldon
setup: brew cargo repos link gitconfig claude-plugins dev-routing sheldon mux
