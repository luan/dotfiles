#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

sudo pacman -Syu --needed --noconfirm git base-devel stow

git clone https://aur.archlinux.org/yay.git /tmp/yay || true
(cd /tmp/yay && makepkg -si --noconfirm)

mkdir $HOME/bin

stow -R alacritty
stow -R autorandr
stow -R compton
stow -R dunst
stow -R fontconfig
stow -R gnupg
stow -R gtk
stow -R home
stow -R i3
stow -R polybar
stow -R rofi
stow -R ssh
stow -R sxiv
stow -R systemd
stow -R wal
stow -R x11
stow -R yay
stow -R zsh
stow -R bin

# yay -S --needed --noconfirm - < packages.txt

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
systemctl --user enable --now wal.timer


