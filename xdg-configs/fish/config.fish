for cert_var in SSL_CERT_FILE CURL_CA_BUNDLE SSL_CERT_DIR NODE_EXTRA_CA_CERTS REQUESTS_CA_BUNDLE
    if set -q $cert_var; and string match -q '/opt/zerobrew/*' -- $$cert_var
        set --erase $cert_var
    end
end

set --export GIT_USERNAME luan

# Some SSH clients, including Termius on iOS, do not send a locale. tmux then
# treats the client as non-UTF-8 and replaces Unicode glyphs with underscores.
# Preserve any locale supplied by local terminals or other SSH clients.
if test -z "$LC_ALL$LC_CTYPE$LANG"
    set --export LANG en_US.UTF-8
    set --export LC_CTYPE en_US.UTF-8
end

# Activate after PATH setup, including shells that inherit MISE_SHELL.
set --export MISE_ACTIVATE_AGGRESSIVE 1

# Optional Homebrew paths. The same config works on macOS, Linuxbrew, and
# machines without Homebrew.
for brew_path in /opt/homebrew/bin /opt/homebrew/sbin /usr/local/bin $HOME/.linuxbrew/bin $HOME/.linuxbrew/sbin
    fish_add_path --path $brew_path
end
if command -sq brew
    eval (brew shellenv fish)
end
fish_add_path --path $HOME/.local/bin $HOME/bin $HOME/.cargo/bin $HOME/.bun/bin $HOME/.opencode/bin

# Fast exit for non-interactive shells (tmux popups, scripts, etc.)
if not status is-interactive
    fish_add_path --path --move $HOME/.local/share/vite-plus/bin
    if command -sq mise
        mise env -s fish | source
    end
    return
end

set --export EDITOR nvim
set --export GIT_EDITOR nvim

# path
fish_add_path /usr/local/go/bin
fish_add_path $HOME/bin
fish_add_path $HOME/.local/bin
fish_add_path $HOME/.emacs.d/bin

set --export XDG_CONFIG_HOME $HOME/.config

# bun
set --export BUN_INSTALL "$HOME/.bun"
fish_add_path --path $BUN_INSTALL/bin

direnv hook fish | source

if status is-interactive
    zoxide init fish | source
    starship init fish | source
end

# Added by OrbStack: command-line tools and integration
# This won't be added again if you remove it.
source ~/.orbstack/shell/init2.fish 2>/dev/null || :

set -Ux CARAPACE_BRIDGES 'zsh,fish,bash,inshellisense' # optional
carapace _carapace | source

# Keep package-manager bins ahead of language-specific bins.
if command -sq brew
    fish_add_path --path --move (brew --prefix)/bin (brew --prefix)/sbin
end
fish_add_path --path --move $HOME/.local/bin

# Added by DeveloperSetup.sh
if test -r "$HOME/.local/share/bcny/config.fish"
    source "$HOME/.local/share/bcny/config.fish"
end

# Vite+ selects the JavaScript runtime and package managers for this directory.
if test -r "$HOME/.config/vite-plus/env.fish"
    source "$HOME/.config/vite-plus/env.fish"
end

if command -sq mise
    mise activate fish | source
    # Reset inherited PATH snapshots after activation.
    set --erase __MISE_ORIG_PATH
    mise hook-env --force -s fish | source
end
