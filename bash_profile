#!/usr/bin/env bash

# Load RVM, if you are using it
[[ -s $HOME/.rvm/scripts/rvm ]] && source $HOME/.rvm/scripts/rvm

# Path to the bash it configuration
export BASH_IT=$HOME/.bash_it

# Lock and Load a custom theme file
# location /.bash_it/themes/
export BASH_IT_THEME='bobby'
export GOPATH=$HOME/code/go
export PATH=$PATH:/usr/local/opt/go/libexec/bin

# Set my editor and git editor
export EDITOR="vim"
export GIT_EDITOR="vim"

# Don't check mail when opening terminal.
unset MAILCHECK

alias git=hub

# Set vcprompt executable path for scm advance info in prompt (demula theme)
# https://github.com/xvzf/vcprompt
#export VCPROMPT_EXECUTABLE=~/.vcprompt/bin/vcprompt

# Load Bash It
source $BASH_IT/bash_it.sh

export LSCOLORS=gxBxhxDxfxhxhxhxhxcxcx
alias ls='ls -G'
