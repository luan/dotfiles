#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

sudo pacman -Syu --needed --noconfirm git yay base-devel stow

stow -R home
stow -R gnupg
stow -R ssh
stow -R yay

yay -S --needed - < packages.txt

mkdir -p ~/.config

clone luan/nvim  .config/nvim  || true
stow -R nvim

clone luan/tmuxfiles .config/tmux || true

(cd $HOME/.config/tmux && ./install)

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

