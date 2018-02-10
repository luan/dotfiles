#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

sudo pacman -Syu --needed --noconfirm yajl git expac

if ! which trizen; then
  pushd "$(mktemp -d)"
    git clone https://aur.archlinux.org/trizen.git
    (cd trizen && makepkg -i --noconfirm)
  popd
fi

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

if [[ "$(getent passwd "$LOGNAME" | cut -d: -f7)" != "$(which zsh)" ]]; then
  sudo chsh -s "$(which zsh)" "$LOGNAME"
fi

"$dotfiles_dir/scripts/load-state"
