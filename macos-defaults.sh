#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Setting macOS system defaults...${NC}"

# Close System Preferences to prevent overriding settings
osascript -e 'tell application "System Preferences" to quit'

# Dock settings
echo -e "${BLUE}Configuring Dock...${NC}"
defaults write com.apple.dock "appswitcher-all-displays" -bool true
defaults write com.apple.dock "autohide" -bool true
defaults write com.apple.dock "autohide-delay" -float 0.0
defaults write com.apple.dock "autohide-time-modifier" -float 0.25
defaults write com.apple.dock "enable-spring-load-actions-on-all-items" -bool true
defaults write com.apple.dock "expose-animation-duration" -float 0.0
defaults write com.apple.dock "launchanim" -bool false
defaults write com.apple.dock "orientation" -string "bottom"
defaults write com.apple.dock "show-process-indicators" -bool true

# Keyboard settings
echo -e "${BLUE}Configuring Keyboard...${NC}"
defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool false
defaults write NSGlobalDomain "InitialKeyRepeat" -int 20
defaults write NSGlobalDomain "KeyRepeat" -int 2
defaults write NSGlobalDomain "NSNavPanelExpandedStateForSaveMode" -bool true
defaults write NSGlobalDomain "NSTextShowsControlCharacters" -bool true
defaults write NSGlobalDomain "AppleKeyboardUIMode" -int 3
defaults write NSGlobalDomain "NSAutomaticSpellingCorrectionEnabled" -bool false

# Finder settings
echo -e "${BLUE}Configuring Finder...${NC}"
defaults write com.apple.finder "AppleShowAllFiles" -bool true
defaults write com.apple.finder "CreateDesktop" -bool false
defaults write com.apple.finder "FXEnableExtensionChangeWarning" -bool false
defaults write com.apple.finder "QuitMenuItem" -bool true
defaults write com.apple.finder "ShowStatusBar" -bool true
defaults write com.apple.finder "_FXShowPosixPathInTitle" -bool false
defaults write NSGlobalDomain "AppleShowAllExtensions" -bool true

# Trackpad settings
echo -e "${BLUE}Configuring Trackpad...${NC}"
defaults write NSGlobalDomain "com.apple.mouse.tapBehavior" -int 1
defaults write NSGlobalDomain "com.apple.trackpad.scaling" -float 3.0

# Misc settings
echo -e "${BLUE}Configuring Miscellaneous settings...${NC}"
defaults write com.apple.LaunchServices "LSQuarantine" -bool false
defaults write NSGlobalDomain "NSAutomaticWindowAnimationsEnabled" -bool false
defaults write NSGlobalDomain "NSWindowResizeTime" -float 0.001
defaults write NSGlobalDomain "NSScrollAnimationEnabled" -bool false
defaults write NSGlobalDomain "AppleShowScrollBars" -string "WhenScrolling"
defaults write NSGlobalDomain "NSWindowShouldDragOnGesture" -bool true

# Restart affected applications
echo -e "${BLUE}Restarting affected applications...${NC}"
for app in "Dock" "Finder" "SystemUIServer"; do
  killall "${app}" &> /dev/null || true
done

echo -e "${GREEN}macOS defaults have been set successfully.${NC}" 