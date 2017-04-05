#!/usr/bin/env bash

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

pac() {
  pacaur -S --noconfirm "$@"
}

sudo pacman -Syu --noconfirm

if ! which pacaur; then
  pushd "$(mktemp -d)"
    git clone https://aur.archlinux.org/cower.git &&
      cd cower && makepkg -i && cd - &&
      git clone https://aur.archlinux.org/pacaur.git &&
      cd pacaur && makepkg -i && cd -
  popd
fi

pac acpi
pac alactritty-git
pac alsa-utils
pac base-devel
pac compton
pac diff-so-fancy
pac direnv
pac dunst
pac fasd
pac fish
pac git-extras
pac grc
pac i3-wm
pac i3blocks-git
pac i3lock
pac jq
pac lastpass-cli
pac powerline-fonts-git
pac rofi
pac stow
pac sysstat
pac tmux
pac ttf-font-awesome
pac ttf-iosevka
pac ttf-iosevka-term
pac ttf-ms-fonts
pac xautolock
pac xclip
pac xdg-utils
pac xsel

stow home
stow i3
stow i3blocks
stow dunst

mkdir -p ~/.tmux/plugins

clone luan/vimfiles    .vim
clone tmux-plugins/tpm .tmux/plugins/tpm

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
sudo chsh -s "$(which fish)" "$user"
fish <(curl -L https://github.com/oh-my-fish/oh-my-fish/raw/master/bin/install) --path=~/.local/share/omf --config=~/.dotfiles/omf/

