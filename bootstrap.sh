#!/usr/bin/env bash

files=" bash_profile
vimrc.after
dir_colors
editrc
gemrc
gitconfig
inputrc
pryrc
tmux.conf
secrets"

for file in $files; do
  mv    $HOME/.$file $HOME/.$file.bak
  ln -s $PWD/$file   $HOME/.$file
done
