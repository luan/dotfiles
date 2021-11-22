#!/usr/bin/env bash

set -e

dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"

is_exec() {
  if ! which "$1" >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

mgr() {
  local mgr='sudo pacman'
  is_exec paru && mgr=paru
  $mgr "$@"
}

install() {
  mgr -S --needed --noconfirm "$@"
}

is_installed() {
  mgr -Qi "$package" >/dev/null 2>&1
}

converge() {
  packages=$(yq "(.[].$1 // [])[]" packages.yaml -r)
  local not_installed=()
  for package in $packages; do
     is_installed "$package" && continue
     not_installed=("${not_installed[@]}" "$package")
  done
  [ "${#not_installed[@]}" -eq 0 ] && return
  install "${not_installed[@]}"
}

ensure_rust() {
  is_exec rust && return
  ! is_installed && install rustup
  rustup update
  rustup default nightly
}

ensure_yq() {
  is_exec yq && return
  install yq
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
  nvim -c 'autocmd User PackerComplete quitall' -c 'PackerSync'
  nvim -c 'UpdateRemotePlugins' -c 'quitall'
  nvim --headless -c 'TSInstallSync all' -c 'quitall'
  nvim -c 'autocmd User PackerComplete quitall' -c 'PackerSync'
}

setup_tmux_config() {
  clone \
    "https://github.com/luan/tmuxfiles" \
    "$HOME/.config/tmux"

  (cd "$HOME/.config/tmux" && ./install)
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
    sudo chsh -s "$(which zsh)" "$LOGNAME"
  fi
}

chaotic_aur() {
  sudo pacman-key --recv-key FBA220DFC880C036 --keyserver keyserver.ubuntu.com
  sudo pacman-key --lsign-key FBA220DFC880C036
  sudo pacman --needed --noconfirm -U 'https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-keyring.pkg.tar.zst' 'https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-mirrorlist.pkg.tar.zst'
}

setup_pacman() {
  chaotic_aur
  sudo rm -f /etc/pacman.conf
  sudo ln -s "$dotfiles_dir/etc/pacman.conf" "/etc/pacman.conf"
  sudo pacman -Sy
}

setup_bin() {
  mkdir -p "$HOME/bin"
  stow -R bin -t "$HOME/bin"
}

enable_services() {
  sudo systemctl enable --now autorandr.service
  sudo systemctl enable --now udisks2.service
}

main() {
  (
  cd "$dotfiles_dir"

  setup_pacman

  ensure_yq
  converge official
  converge chaotic

  ensure_rust
  converge aur

  change_shell
  setup_nvim_config
  setup_tmux_config
  setup_gitconfig
  stow -R xdg-configs -t "$HOME/.config"
  setup_bin
  stow -R home -t "$HOME"
  stow -R x11  -t "$HOME"

  mgr -Syu --noconfirm
  )
}

main

