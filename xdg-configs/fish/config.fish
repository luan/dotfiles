# Fast exit for non-interactive shells (tmux popups, scripts, etc.)
if not status is-interactive
    set --export PATH /opt/homebrew/bin /usr/local/bin $HOME/bin $HOME/.local/bin $PATH
    return
end

# aliases
alias vim=nvim
alias ls="eza --icons"
alias ll="eza --icons -l"
alias la="eza --icons -la"
alias lt="eza --icons --tree"

set --export EDITOR vim
set --export GIT_EDITOR vim

# sccache — compiler cache for C/C++
set --export CMAKE_C_COMPILER_LAUNCHER sccache
set --export CMAKE_CXX_COMPILER_LAUNCHER sccache


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

# opencode
fish_add_path /Users/luan/.opencode/bin

