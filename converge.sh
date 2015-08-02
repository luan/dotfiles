#!/bin/bash

copy_plist() {
  name=$1
  cp ~/.dotfiles/osx/plists/$name ~/Library/Preferences/$name
}

# Disable Caps Lock
# Install Xcode - https://itunes.apple.com/us/app/xcode/id497799835?mt=12
xcode-select --install

# Install Homebrew and enable cask and taps
ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
brew install caskroom/cask/brew-cask
brew tap caskroom/versions
brew tap pivotal/tap
brew tap universal-ctags/universal-ctags
brew tap git-duet/tap

# general dependencies
brew install git
git clone https://github.com/luan/atom-config.git ~/.atom
git clone https://github.com/luan/vimfiles ~/.vim
git clone https://github.com/luan/dotfiles ~/.dotfiles
git clone --depth=1 https://github.com/Bash-it/bash-it.git ~/.bash_it

# apps and configs

brew cask install karabiner
copy_plist org.pqrs.Karabiner.plist

brew cask install google-chrome

brew cask install iterm2-beta
copy_plist com.googlecode.iterm2.plist

brew cask install spectacle
copy_plist com.divisiblebyzero.Spectacle.plist

mkdir -p $HOME/Library/Fonts
pushd $HOME/Library/Fonts
curl -o "Meslo LG S Regular for Powerline.otf" https://github.com/powerline/fonts/raw/master/Meslo/Meslo%20LG%20S%20Regular%20for%20Powerline.otf

brew install ack ag aria2 bash-completion chruby cloudfoundry-cli direnv \
  fasd fzf git-duet htop-osx jq libevent libffi libtool libyaml mercurial \
  ncdu node pstree python ruby-install tig tmate tmux tree watch wget xz

# formulas that need setup
brew install mysql && \
  ln -sfv /usr/local/opt/mysql/*.plist ~/Library/LaunchAgents

brew install postgres && \
  ln -sfv /usr/local/opt/postgresql/*.plist ~/Library/LaunchAgents

# formulas that need customization
brew install vim --with-lua
brew install go --with-cc-common
brew install macvim --with-lua
brew install universal-ctags --HEAD

cd $HOME/.dotfiles
./bootstrap.sh
./osx/setup-preferences

~/.vim/install

sudo vim /etc/shells +'norm 5ggO/usr/local/bin/bash' +wq

exec bash -l
echo chruby chruby-auto fzf fasd ssh tmux osx | xargs -n1 echo bash-it enable plugin | bash -l
echo bash-it brew defaults gem git gulp npm packer pip rake ssh tmux vagrant | xargs -n1 echo bash-it enable completion | bash -l

brew cask install alfred
brew cask install slack
brew cask install sublime-text3
brew cask install atom
brew cask install virtualbox
brew cask install vagrant
brew cask install wraparound

go get github.com/vito/boosh
go get github.com/vito/spiff
go get github.com/tools/godep

pip install aws

ruby-install ruby 2.1.6
chruby ruby-2.1.6
gem install bosh_cli
gem install bundler
