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

bosh() {
  (
    if [ ! -f $HOME/workspace/fast-bosh/Gemfile ]; then
      mkdir -p $HOME/workspace/fast-bosh
      pushd $HOME/workspace/fast-bosh
      echo -e "source 'https://rubygems.org'\ngem 'bosh_cli'" > Gemfile
      gem install bundler
      bundle install
      popd
    fi
    GEM_PATH= BUNDLE_GEMFILE=$HOME/workspace/fast-bosh/Gemfile bundle exec bosh "$@"
  )
}
export -f bosh

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
