#!/usr/bin/env bash

export SCM_CHECK=true
export BASH_IT=$HOME/.bash_it
export BASH_IT_THEME='bobby'
source $BASH_IT/bash_it.sh

export EDITOR='vim'
export GIT_EDITOR='vim'

export GIT_DUET_GLOBAL=true
export GIT_DUET_ROTATE_AUTHOR=1

unset MAILCHECK

export GOPATH=$HOME/workspace/go
export PATH=$GOPATH/bin:$PATH

source $HOME/.vim/scripts/base16-shell/base16-monokai.dark.sh

alias ls='ls -G'

bosh() {
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
