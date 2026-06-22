for cert_var in SSL_CERT_FILE CURL_CA_BUNDLE SSL_CERT_DIR NODE_EXTRA_CA_CERTS REQUESTS_CA_BUNDLE
    if set -q $cert_var; and string match -q '/opt/zerobrew/*' -- $$cert_var
        set --erase $cert_var
    end
end

# Fast exit for non-interactive shells (tmux popups, scripts, etc.)
if not status is-interactive
    set --export PATH $HOME/.local/share/mise/shims /opt/homebrew/bin /opt/homebrew/sbin $HOME/bin $HOME/.local/bin /usr/local/bin $PATH
    return
end

# aliases
alias vim=nvim
alias ls="eza --icons"
alias ll="eza --icons -l"
alias la="eza --icons -la"
alias lt="eza --icons --tree"
alias gs=git-spice

set --export EDITOR nvim
set --export GIT_EDITOR nvim

# sccache — compiler cache for C/C++
if not test -f ~/.config/fish/.no-sccache
    set --export CMAKE_C_COMPILER_LAUNCHER sccache
    set --export CMAKE_CXX_COMPILER_LAUNCHER sccache
end

eval "$(/opt/homebrew/bin/brew shellenv)"

# path
fish_add_path /usr/local/bin
fish_add_path /usr/local/go/bin
fish_add_path $HOME/bin
fish_add_path $HOME/.local/bin
fish_add_path $HOME/.emacs.d/bin

set --export XDG_CONFIG_HOME $HOME/.config

# bun
set --export BUN_INSTALL "$HOME/.bun"
set --export PATH $BUN_INSTALL/bin $PATH

direnv hook fish | source

if status is-interactive
    zoxide init fish | source
    starship init fish | source
end

# Added by OrbStack: command-line tools and integration
# This won't be added again if you remove it.
source ~/.orbstack/shell/init2.fish 2>/dev/null || :

alias view="nvim -R"

COMPLETE=fish jj | source

set -Ux CARAPACE_BRIDGES 'zsh,fish,bash,inshellisense' # optional
carapace _carapace | source
if command -sq cargo
    source $__fish_data_dir/completions/cargo.fish # carapace's cargo completer is broken for -p/--package
end

# opencode
fish_add_path $HOME/.opencode/bin

# Keep mise shims ahead of language/package-manager bins added above.
fish_add_path --path --move /opt/homebrew/bin /opt/homebrew/sbin
fish_add_path --path --move $HOME/.local/share/mise/shims
