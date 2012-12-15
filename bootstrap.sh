#!/usr/bin/env bash

mv $HOME/.bash_profile $HOME/.bash_profile.bak
mv $HOME/.vimrc.after $HOME/.vimrc.after.bak
mv $HOME/.dirs $HOME/.dirs.bak
mv $HOME/.editrc $HOME/.editrc.bak
mv $HOME/.gemrc $HOME/.gemrc.bak
mv $HOME/.gitconfig $HOME/.gitconfig.bak
mv $HOME/.inputrc $HOME/.inputrc.bak
mv $HOME/.pryrc $HOME/.pryrc.bak
mv $HOME/.secrets $HOME/.secrets.bak

ln -s `pwd`/bash_profile $HOME/.bash_profile
ln -s `pwd`/vimrc.after $HOME/.vimrc.after
ln -s `pwd`/dirs $HOME/.dirs
ln -s `pwd`/editrc $HOME/.editrc
ln -s `pwd`/gemrc $HOME/.gemrc
ln -s `pwd`/gitconfig $HOME/.gitconfig
ln -s `pwd`/inputrc $HOME/.inputrc
ln -s `pwd`/pryrc $HOME/.pryrc
ln -s `pwd`/secrets $HOME/.secrets
