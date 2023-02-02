#!/usr/bin/env bash

set -e

is_exec() {
  if ! which "$1" >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

clone() {
  local src="$1"
  local dst="$2"

  set +e
  git clone "$src" "$dst" 2>/dev/null
  set -e
}

setup_nvim_config() {
  clone \
    "https://github.com/luan/nvim" \
    "$HOME/.config/nvim"
}

setup_tmux_config() {
  clone \
    "https://github.com/luan/tmuxfiles" \
    "$HOME/.config/tmux"
  (cd "$HOME/.config/tmux" && ./install)
}

setup_zshrc() {
  if ! grep --quiet "path=$dotfiles_dir/zsh/zshrc" "$HOME/.zshrc"; then
  cat << EOF >> "$HOME/.zshrc"
source "$dotfiles_dir/zsh/zshrc"
EOF
  fi
}


setup_gitconfig() {
  if ! grep --quiet "path=$dotfiles_dir/gitconfig" "$HOME/.gitconfig"; then
  cat << EOF >> "$HOME/.gitconfig"

[include]
  path=$dotfiles_dir/gitconfig
EOF
  else
    echo "Skipping gitconfig"
  fi
}

change_shell() {
  if [[ "$(getent passwd "$LOGNAME" | cut -d: -f7)" != "$(which zsh)" ]]; then
    sudo chsh -s "$(which zsh)" "$LOGNAME" || true
  fi
}

setup_bin() {
  mkdir -p "$HOME/bin"
  stow -R bin -t "$HOME/bin"
}

_current_os=$(uname)

is_macos() {
  [[ "$_current_os" == "Darwin" ]]
}

is_linux() {
  [[ "$_current_os" == "Linux" ]]
}
