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

# Install Homebrew-managed casks, services, and fallback formulae from Brewfile
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

    HOMEBREW_ALLOW_FORMULA=1 "$real_brew" bundle --file="{{ dotfiles_dir }}/Brewfile"

# Install or update zerobrew without letting its installer edit shell config
zerobrew:
    #!/usr/bin/env bash
    set -euo pipefail

    export ZEROBREW_BIN="$HOME/.local/bin"
    export ZEROBREW_ROOT="/opt/zerobrew"
    export ZEROBREW_PREFIX="/opt/zerobrew"

    mkdir -p "$ZEROBREW_BIN"
    curl -fsSL https://zerobrew.rs/install | bash -s -- --no-modify-path
    "$ZEROBREW_BIN/zb" --version

# Install zerobrew-managed CLI formulae from Brewfile.zerobrew
zerobrew-packages: zerobrew
    #!/usr/bin/env bash
    set -euo pipefail

    zb_bin="$HOME/.local/bin/zb"
    export PATH="$HOME/.local/bin:/opt/zerobrew/bin:$PATH"

    formulas=()
    while IFS= read -r formula; do
        formulas+=("$formula")
    done < <(ruby -ne 'puts $1 if /^brew ["'"'"']([^"'"'"']+)["'"'"']/' "{{ dotfiles_dir }}/Brewfile.zerobrew")

    install_formula() {
        local formula="$1"
        local output links link target attempt
        echo "==> zerobrew install $formula"

        for attempt in 1 2 3 4 5; do
            if output="$($zb_bin install "$formula" 2>&1)"; then
                printf '%s\n' "$output"
                return 0
            fi

            if ! grep -q 'link conflict' <<<"$output"; then
                printf '%s\n' "$output"
                return 1
            fi

            echo "==> Repairing stale zerobrew symlinks for $formula (attempt $attempt)"
            mapfile -t links < <(grep -Eo "'/opt/zerobrew/[^']+'" <<<"$output" | tr -d "'" | sort -u)
            if [ "${#links[@]}" -eq 0 ]; then
                printf '%s\n' "$output"
                return 1
            fi
            for link in "${links[@]}"; do
                [ -L "$link" ] || continue
                target="$(readlink "$link")"
                case "$target" in
                    /opt/zerobrew/Cellar/*) rm "$link" ;;
                    *) echo "refusing to remove non-zerobrew link $link -> $target" >&2; return 1 ;;
                esac
            done
        done

        echo "zerobrew install $formula still has link conflicts after 5 repair attempts" >&2
        return 1
    }

    for formula in "${formulas[@]}"; do
        install_formula "$formula"
    done

    doctor_clean=0
    for attempt in 1 2 3; do
        doctor_output="$($zb_bin doctor 2>&1)"
        if grep -q 'No issues found' <<<"$doctor_output"; then
            doctor_clean=1
            break
        fi
        if ! grep -q 'Run zb doctor --repair' <<<"$doctor_output"; then
            printf '%s\n' "$doctor_output"
            exit 1
        fi
        echo "==> Repairing zerobrew doctor issues (attempt $attempt)"
        $zb_bin doctor --repair
    done
    if [ "$doctor_clean" -ne 1 ]; then
        $zb_bin doctor
        exit 1
    fi
    echo "✓ zerobrew doctor passed"

# Replace stale local Qt resource compiler with the zerobrew-managed one
zerobrew-rcc: zerobrew-packages
    #!/usr/bin/env bash
    set -euo pipefail

    rcc_src="/opt/zerobrew/opt/qt@5/bin/rcc"
    rcc_dst="$HOME/.local/bin/rcc"

    if [ ! -x "$rcc_src" ]; then
        echo "✗ zerobrew qt@5 rcc not found at $rcc_src" >&2
        exit 1
    fi

    mkdir -p "$HOME/.local/bin"
    if [ "$(readlink "$rcc_dst" 2>/dev/null || true)" = "$rcc_src" ]; then
        echo "✓ rcc already points to zerobrew qt@5"
    else
        if [ -e "$rcc_dst" ] || [ -L "$rcc_dst" ]; then
            backup_dir="${XDG_STATE_HOME:-$HOME/.local/state}/dotfiles/homebrew-linkage-backups"
            mkdir -p "$backup_dir"
            backup="$backup_dir/rcc.$(date +%Y%m%d%H%M%S)"
            mv "$rcc_dst" "$backup"
            echo "→ Backed up existing rcc to $backup"
        fi
        ln -s "$rcc_src" "$rcc_dst"
        echo "✓ Linked $rcc_dst -> $rcc_src"
    fi

    if otool -L "$rcc_dst" | grep -q '/opt/homebrew'; then
        echo "✗ rcc still links to Homebrew libraries" >&2
        exit 1
    fi
    "$rcc_dst" -v

# Report executables that still link against Homebrew libraries before any pruning
homebrew-linkage-audit:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v otool >/dev/null; then
        echo "✗ otool not found; install Xcode Command Line Tools first" >&2
        exit 1
    fi

    found=0
    for root in "$HOME/bin" "$HOME/.local/bin" "/opt/zerobrew/bin"; do
        [ -d "$root" ] || continue
        while IFS= read -r -d '' file; do
            if ! file "$file" 2>/dev/null | grep -q 'Mach-O'; then
                continue
            fi
            refs="$(otool -L "$file" 2>/dev/null | grep -E '/opt/homebrew|/usr/local/(opt|Cellar)' || true)"
            if [ -n "$refs" ]; then
                found=1
                printf '\n%s\n%s\n' "$file" "$refs"
            fi
        done < <(find "$root" -type f -perm -111 -print0)
    done

    if [ "$found" -eq 0 ]; then
        echo "✓ No Homebrew library references found in scanned executable paths"
    else
        printf '\n⚠ Homebrew library references remain; do not prune those Homebrew formulae yet\n' >&2
        exit 1
    fi

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
    PATH="$HOME/.local/bin:/opt/zerobrew/bin:$PATH" sheldon --config-file "{{ dotfiles_dir }}/xdg-configs/sheldon/plugins.toml" lock --update

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
    export PATH="$HOME/.local/bin:/opt/zerobrew/bin:$PATH"

    if ! command -v cargo-binstall &>/dev/null; then
        echo "⚠ cargo-binstall not found, run 'just zerobrew-packages' first"
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

# Full setup: zerobrew, Homebrew fallbacks/casks, cargo, repos, link, gitconfig, claude-plugins, dev-routing, mux, sheldon
setup: zerobrew-rcc brew cargo repos link gitconfig claude-plugins dev-routing sheldon mux
