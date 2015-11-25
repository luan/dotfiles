#!/usr/bin/env bash

export EDITOR='vim'
export GIT_EDITOR='vim'

export GIT_DUET_GLOBAL=true
export GIT_DUET_ROTATE_AUTHOR=true

export GOPATH=$HOME/workspace/go
export PATH=$GOPATH/bin:$PATH

bash $HOME/.vim/scripts/base16-shell/base16-monokai.dark.sh

export BOSH_USE_BUNDLER=true

export NVIM_TUI_ENABLE_TRUE_COLOR=1
alias vim=nvim

source /usr/local/share/chruby/chruby.fish
chruby 2.1.7

