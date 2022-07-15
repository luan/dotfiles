#!/usr/bin/env bash

require() {
  dotfiles_dir="$(cd "$(dirname "$0")" && pwd)"
  source "${dotfiles_dir}/lib/$@"
}

require 'common.sh'
require 'stdout.sh'
require 'macos.sh'

main() {
  dotheader "Setting up stuff on @umacOS"
  if ! is_macos; then
    dotsay "@redBut this isn't a @bMac!@reset Exiting... "
    exit 1
  fi

  brew_install_all "$(cat "${dotfiles_dir}/mac-packages.txt")"
  mac_tweaks
  setup_nvim_config
  setup_gitconfig
  setup_bin
}

main
