#!/usr/bin/env bash

# Path to the bash it configuration
export BASH_IT=$HOME/.bash_it

# Lock and Load a custom theme file
# location /.bash_it/themes/
export BASH_IT_THEME='bobby'

# Set my editor and git editor
export EDITOR='vim'
export GIT_EDITOR='vim'
export GIT_DUET_GLOBAL=true
export GIT_DUET_ROTATE_AUTHOR=1

# Don't check mail when opening terminal.
unset MAILCHECK

# Change this to your console based IRC client of choice.

export IRC_CLIENT='irssi'

# Set this to false to turn off version control status checking within the prompt for all themes
export SCM_CHECK=true

# Set vcprompt executable path for scm advance info in prompt (demula theme)
# https://github.com/xvzf/vcprompt
#export VCPROMPT_EXECUTABLE=~/.vcprompt/bin/vcprompt

export GOPATH=$HOME/workspace/go
export PATH=$GOPATH/bin:$PATH

# Load Bash It
source $BASH_IT/bash_it.sh
source $HOME/.vim/scripts/base16-shell/base16-monokai.dark.sh
alias ls='ls -G'

function bosh() {
  (
    BUNDLE_GEMFILE=~/workspace/fast-bosh/Gemfile bundle exec bosh "$@"
  )
}
export -f bosh

_direnv_hook() {
  eval "$(direnv export bash)";
};
if ! [[ "$PROMPT_COMMAND" =~ _direnv_hook ]]; then
  PROMPT_COMMAND="_direnv_hook;$PROMPT_COMMAND";
fi

source /Users/luan/.iterm2_shell_integration.bash
export PATH="/usr/local/sbin:$PATH"
