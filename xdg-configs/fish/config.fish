# aliases
alias vim=nvim

set --export EDITOR vim
set --export GIT_EDITOR vim

eval "$(/opt/homebrew/bin/brew shellenv)"

# path
set --export PATH /usr/local/bin $PATH
set --export PATH /usr/local/go/bin $PATH
set --export PATH $HOME/bin $PATH
set --export PATH $HOME/.local/bin $PATH
set --export PATH $HOME/.emacs.d/bin $PATH

set --export XDG_CONFIG_HOME $HOME/.config

# pnpm (must come before npm)
set -gx PNPM_HOME /Users/luan/Library/pnpm
if not string match -q -- $PNPM_HOME $PATH
    set -gx PATH "$PNPM_HOME" $PATH
end

# npm
set --export PATH $(npm config get prefix)/bin $PATH

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

# opencode
fish_add_path /Users/luan/.opencode/bin

# git-grove shell integration
source /Users/luan/.local/share/git-grove/git-grove-auto.fish

alias claude="/Users/luan/.claude/local/claude"
