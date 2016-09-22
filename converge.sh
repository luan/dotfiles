#!/usr/bin/env bash

copy_plist() {
  name=$1
  cp $HOME/.dotfiles/osx/plists/$name $HOME/Library/Preferences/$name
}

symlink_dotfiles() {
  set +e
  for file in $@; do
    ln -fs $HOME/.dotfiles/$file $HOME/.$file
  done
  set -e
}

clone() {
  set +e
  git clone https://github.com/$1 $HOME/$2
  set -e
}

hostname=$1
if [ -z "$hostname" ]; then
  echo "usage: ./converge.sh <hostname>"
  exit 1
fi

# Enable passwordless sudo
sudo perl -pi -e 's/%admin\s+ALL=\(ALL\)\s+ALL/%admin ALL=(ALL) NOPASSWD: ALL/' /etc/sudoers

sudo scutil --set LocalHostName $hostname
sudo scutil --set ComputerName $hostname
sudo scutil --set HostName $hostname

# Disable Caps Lock (manual step :/)

# Install Xcode - https://itunes.apple.com/us/app/xcode/id497799835?mt=12
if [ ! -d /Applications/Xcode.app/ ]; then
  open https://itunes.apple.com/us/app/xcode/id497799835?mt=12
  echo "press enter when done"
  read
fi

xcode-select --install

# Install Homebrew and enable cask and taps
ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"

set -e

brew update
brew tap Homebrew/bundle
brew bundle
brew cleanup --force

clone luan/atom-config .atom
clone luan/vimfiles    .vim
clone luan/dotfiles    .dotfiles

mkdir -p ~/.tmux/plugins
clone tmux-plugins/tpm .tmux/plugins/tpm

# configs
copy_plist org.pqrs.Karabiner.plist
copy_plist com.googlecode.iterm2.plist
copy_plist com.divisiblebyzero.Spectacle.plist

pip install --upgrade pip
pip3 install --upgrade pip
pip3 install neovim

cd $HOME/.dotfiles
symlink_dotfiles vimrc.local vimrc.local.before dir_colors \
  editrc gemrc gitconfig inputrc pryrc tmux.conf

./osx/setup-preferences

mkdir -p $HOME/workspace/go
export GOPATH=$HOME/workspace/go

$HOME/.vim/update

cat <<EOF | sudo tee /etc/shells
# List of acceptable shells for chpass(1).
# Ftpd will not allow users to connect who are not using
# one of these shells.

/usr/local/bin/fish
/usr/local/bin/bash
/bin/bash
/bin/csh
/bin/ksh
/bin/sh
/bin/tcsh
/bin/zsh
EOF

user=$(whoami)
sudo chsh -s /usr/local/bin/fish $user
fish <(curl -L https://github.com/oh-my-fish/oh-my-fish/raw/master/bin/install) --path=~/.local/share/omf --config=~/.dotfiles/omf/

go get -v -u github.com/vito/boosh
go get -v -u github.com/tools/godep

curl -L  -o /tmp/spiff.zip https://github.com/cloudfoundry-incubator/spiff/releases/download/v1.0.7/spiff_darwin_amd64.zip
mkdir -p $HOME/bin
unzip -o /tmp/spiff.zip -d $HOME/bin

gpg --keyserver hkp://keys.gnupg.net --recv-keys 409B6B1796C275462A1703113804BB82D39DC0E3
curl -sSL https://get.rvm.io | bash -s stable --ruby --gems=bundler,bosh_cli
curl -L --create-dirs -o ~/.config/fish/functions/rvm.fish https://raw.github.com/lunks/fish-nuggets/master/functions/rvm.fish

exec fish -l
