#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

sudo pacman -Syu --needed --noconfirm yajl git expac

stow home
stow x11
stow autorandr
stow i3
stow dunst
stow compton
stow alacritty
stow rofi
stow polybar

mkdir -p ~/.tmux

clone luan/vimfiles    .config/vim
clone luan/tmuxfiles   .config/tmux

mkdir -p "$HOME/workspace/go"
export GOPATH="$HOME/workspace/go"

if ! grep --quiet "path=$dotfiles_dir/gitconfig" "$HOME/.gitconfig"; then
cat << EOF >> "$HOME/.gitconfig"

[include]
  path=$dotfiles_dir/gitconfig
EOF
else
  echo "Skipping gitconfig"
fi


user=$(whoami)
sudo chsh -s "$(which zsh)" "$user"

