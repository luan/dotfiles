# aliases
alias vim=nvim

set --export EDITOR vim
set --export GIT_EDITOR vim

eval "$(/opt/homebrew/bin/brew shellenv)"

# path
fish_add_path /usr/local/bin
fish_add_path /usr/local/go/bin
fish_add_path $HOME/bin
fish_add_path $HOME/.local/bin
fish_add_path $HOME/.emacs.d/bin

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
