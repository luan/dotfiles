#!/usr/bin/env bash

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

symlink_dotfiles() {
  set +e
  for file in "$@"; do
    ln -fs "$HOME/.dotfiles/$file" "$HOME/.$file"
  done
  set -e
}

clone() {
  set +e
  git clone "https://github.com/$1" "$HOME/$2"
  set -e
}

mkdir -p ~/.tmux/plugins

clone luan/vimfiles    .vim
clone tmux-plugins/tpm .tmux/plugins/tpm

cd "$dotfiles_dir"
symlink_dotfiles \
  dir_colors \
  gemrc \
  git-authors \
  inputrc \
  tmux.conf \
  vimrc.local \
  vimrc.local.before

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

# "$HOME/.vim/update"

