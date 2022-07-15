#!/usr/bin/env bash

mac_tweaks() {
  defaults write com.apple.LaunchServices LSQuarantine -bool false
  defaults write NSGlobalDomain NSAutomaticWindowAnimationsEnabled -bool false
  defaults write com.apple.dock expose-animation-duration -int 0
  defaults write com.apple.finder QuitMenuItem -bool true
  defaults write NSGlobalDomain AppleShowAllExtensions -bool true
  defaults write com.apple.finder AppleShowAllFiles -string "YES"
  defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true
  defaults write com.apple.finder FXEnableExtensionChangeWarning -bool false
  chflags nohidden ~/Library
  defaults write com.apple.dock show-process-indicators -bool true
  defaults write com.apple.dock autohide-time-modifier -float 0.25
  defaults write com.apple.dock autohide -bool true
  defaults write com.apple.dock autohide-time-modifier -float 0
  defaults write com.apple.dock autohide-delay -float 0
}

brew_install() {
  local package=$1

  ! is_macos && return 1

  if brew list "$package" > /dev/null 2>&1; then
    dotsay "+ $package already installed... skipping."
  else
    brew install $@
  fi
}

brew_install_all() {
  ! is_macos && return 1

  local packages="$(echo -e "$@" | sort)"
  local installed_packages="$(brew list -1 | sort)"

  for package in $(comm -23 <(echo -e "${packages}") <(echo -e "${installed_packages}")); do
    brew_install $package
  done
}
