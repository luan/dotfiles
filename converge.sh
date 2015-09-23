#!/usr/bin/env bash

copy_plist() {
  name=$1
  cp $HOME/.dotfiles/osx/plists/$name $HOME/Library/Preferences/$name
}

symlink_dotfiles() {
  set +e
  for file in $@; do
    ln -s $HOME/.dotfiles/$file $HOME/.$file
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

# Disable Caps Lock
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
clone Bash-it/bash-it  .bash_it

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

brew_upinstall ack ag aria2 bash-completion chruby cloudfoundry-cli direnv \
  fasd fzf git-duet htop-osx jq libevent libffi libtool libyaml mercurial \
  ncdu pstree ruby-install tig tmate tmux tree watch wget xz

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
symlink_dotfiles bash_profile vimrc.local vimrc.local.before dir_colors \
  editrc gemrc gitconfig inputrc pryrc tmux.conf

./osx/setup-preferences

$HOME/.vim/install

sudo vim /etc/shells +'norm 5ggO/usr/local/bin/bash' +wq

set +e
source $HOME/.bash_profile
set -e

rm -rf $HOME/.bash_it/plugins/enabled/*
rm -rf $HOME/.bash_it/completion/enabled/*
rm -rf $HOME/.bash_it/aliases/enabled/*
echo chruby fasd ssh tmux osx | xargs -n1 echo bash-it enable plugin | bash -l
echo bash-it brew defaults gem git gulp npm packer pip rake ssh tmux vagrant | xargs -n1 echo bash-it enable completion | bash -l

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

brew install awscli

ruby-install ruby 2.1.7 --no-reinstall
chruby ruby-2.1.7
gem install bosh_cli
gem install bundler

mkdir -p $HOME/workspace
mkdir -p $HOME/deployments/{concourse,bosh-lite}

set +e
source $HOME/.bash_profile
set -e
