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

brew_install() {
  set +e
  brew install $@
  set -e
}

brew_upinstall() {
  brew install $@ || brew upgrade $@
}

# Enable passwordless sudo
sudo perl -pi -e 's/%admin\s+ALL=\(ALL\)\s+ALL/%admin ALL=(ALL) NOPASSWD: ALL/' /etc/sudoers

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
brew_upinstall caskroom/cask/brew-cask
brew tap caskroom/versions
brew tap caskroom/fonts
brew tap pivotal/tap
brew tap universal-ctags/universal-ctags
brew tap git-duet/tap
brew tap nviennot/tmate
brew tap neovim/neovim

# general dependencies
brew_upinstall git

clone luan/atom-config .atom
clone luan/vimfiles    .vim
clone luan/dotfiles    .dotfiles

# apps and configs
brew cask install karabiner
copy_plist org.pqrs.Karabiner.plist

brew cask install google-chrome

brew cask install iterm2-beta
copy_plist com.googlecode.iterm2.plist

brew cask install spectacle
copy_plist com.divisiblebyzero.Spectacle.plist

brew cask install font-meslo-lg-for-powerline
brew cask install font-fira-mono-for-powerline

brew_upinstall ack ag aria2 rvm loudfoundry-cli direnv fasd fish fzf
brew_upinstall bash git-duet htop-osx jq libevent libffi libtool libyaml mercurial
brew_upinstall ncdu pstree ruby-install tig tmate tmux tree watch wget xz

brew_install node python python3

# formulas that need setup
brew_install mysql && \
  ln -sfv /usr/local/opt/mysql/*.plist $HOME/Library/LaunchAgents

brew_install postgres && \
  ln -sfv /usr/local/opt/postgresql/*.plist $HOME/Library/LaunchAgents

# formulas that need customization
brew_upinstall vim --with-lua
brew_install go --with-cc-common
brew_upinstall macvim --with-lua
brew_upinstall universal-ctags --HEAD

brew_install neovim --HEAD
sudo pip3 install neovim

cd $HOME/.dotfiles
symlink_dotfiles vimrc.local vimrc.local.before dir_colors \
  editrc gemrc gitconfig inputrc pryrc tmux.conf

./osx/setup-preferences

$HOME/.vim/install

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

chsh -s /usr/local/bin/fish
OMF_CONFIG=$HOME/.dotfiles/omf CI=true fish <(curl -L https://github.com/oh-my-fish/oh-my-fish/raw/master/bin/install) || true

brew cask install alfred
brew cask install slack
brew cask install sublime-text3
brew cask install atom

echo "VritualBox installation may fail if you have VMs running, consider powering them off."
brew cask install virtualbox

brew cask install vagrant
brew cask install wraparound

go get -v -u github.com/vito/boosh
go get -v -u github.com/tools/godep

curl -L  -o /tmp/spiff.zip https://github.com/cloudfoundry-incubator/spiff/releases/download/v1.0.7/spiff_darwin_amd64.zip
mkdir -p $HOME/bin
unzip -o /tmp/spiff.zip -d $HOME/bin

brew_upinstall awscli

bash <<EOF
  source /usr/local/opt/chruby/share/chruby/chruby.sh
  ruby-install ruby 2.1.7 --no-reinstall
  chruby ruby-2.1.7
  gem install bosh_cli
  gem install bundler
EOF

mkdir -p $HOME/workspace
mkdir -p $HOME/workspace/concourse-lite

(cd $HOME/workspace/concourse-lite && vagrant init concourse/lite || true )

exec fish -l
