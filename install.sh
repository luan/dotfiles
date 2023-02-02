#!/usr/bin/env bash

require() {
  dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"
  source "${dotfiles_dir}/lib/$@"
}

require 'common.sh'
require 'stdout.sh'
require 'macos.sh'

main() {
  (
  cd "$dotfiles_dir"

  if is_macos; then
    dotheader "Setting up stuff on @umacOS"
    brew_install_all "$(cat "${dotfiles_dir}/mac-packages.txt")"
    dotsay "@redBut this isn't a @bMac!@reset Exiting... "
    mac_tweaks
    exit 1
  fi

  change_shell
  setup_nvim_config
  setup_tmux_config
  setup_gitconfig
  stow -R xdg-configs -t "$HOME/.config"
  stow -R home -t "$HOME"
  setup_bin
  )
}

main
