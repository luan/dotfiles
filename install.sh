#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

sudo pacman -Syu --needed --noconfirm git base-devel stow

git clone https://aur.archlinux.org/yay.git /tmp/yay
(cd /tmp/yay && makepkg -si --noconfirm)

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

mkdir -p "$HOME/workspace"
export GOPATH="$HOME/workspace"

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

rustup default stable

sudo systemctl enable --now udisks2.service

chmod 700 ~/.gnupg
curl https://keybase.io/cfcluan/pgp_keys.asc | gpg --import
