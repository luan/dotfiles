#!/usr/bin/env bash

export SCM_CHECK=true
export BASH_IT=$HOME/.bash_it
export BASH_IT_THEME='bobby'
source $BASH_IT/bash_it.sh

export EDITOR='vim'
export GIT_EDITOR='vim'

export GIT_DUET_GLOBAL=true
export GIT_DUET_ROTATE_AUTHOR=true

unset MAILCHECK

export GOPATH=$HOME/workspace/go
export PATH=$GOPATH/bin:$PATH

source $HOME/.vim/scripts/base16-shell/base16-monokai.dark.sh

alias ls='ls -G'

export BOSH_USE_BUNDLER=true

_direnv_hook() {
  eval "$(direnv export bash)";
};

if ! [[ "$PROMPT_COMMAND" =~ _direnv_hook ]]; then
  PROMPT_COMMAND="_direnv_hook;$PROMPT_COMMAND";
fi

export PATH="$HOME/bin:/usr/local/sbin:$PATH"

if [ -f $(brew --prefix)/etc/bash_completion ]; then
  . $(brew --prefix)/etc/bash_completion
fi

export NVIM_TUI_ENABLE_TRUE_COLOR=1
alias vim=nvim

[[ -s "$HOME/.rvm/scripts/rvm" ]] && source "$HOME/.rvm/scripts/rvm" # Load RVM into a shell session *as a function*
