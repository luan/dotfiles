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

stow -R home
stow -R x11
stow -R autorandr
stow -R i3
stow -R dunst
stow -R compton
stow -R alacritty
stow -R rofi
stow -R polybar
stow -R sxiv

mkdir -p ~/.config

clone luan/nvim  .config/nvim  || true
clone luan/tmuxfiles .config/tmux || true

(cd $HOME/.config/tmux && ./install)
(cd $HOME/.config/vim  && ./install)

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
