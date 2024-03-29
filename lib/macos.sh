#!/usr/bin/env bash

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

mac_tweaks() {
  if [[ "$(uname)" != "Darwin" ]]; then
    echo "Not on MacOS, exiting..."
    exit 1
  fi

  # Turbo key repeat
  defaults write NSGlobalDomain ApplePressAndHoldEnabled -bool false

  # Anything lower than 15 seems too fast.
  defaults write -g InitialKeyRepeat -int 150

  # You can go as low as 1, but that's too darn fast for me.
  defaults write -g KeyRepeat -int 40

  # Show macOS app switcher across all monitors
  defaults write com.apple.Dock appswitcher-all-displays -bool true

  # Expand save panel by default
  defaults write NSGlobalDomain NSNavPanelExpandedStateForSaveMode -bool true

  # Disable the “Are you sure you want to open this application?” dialog
  defaults write com.apple.LaunchServices LSQuarantine -bool false

  # Display ASCII control characters using caret notation in standard text views
  # Try e.g. `cd /tmp; unidecode "\x{0000}" > cc.txt; open -e cc.txt`
  defaults write NSGlobalDomain NSTextShowsControlCharacters -bool true

  # Disable opening and closing window animations
  defaults write NSGlobalDomain NSAutomaticWindowAnimationsEnabled -bool false

  # Disable Mission Control Animations
  defaults write com.apple.dock expose-animation-duration -int 0

  # Increase window resize speed for Cocoa applications
  defaults write NSGlobalDomain NSWindowResizeTime -float 0.001

  # Enable full keyboard access for all controls (e.g. enable Tab in modal dialogs)
  defaults write NSGlobalDomain AppleKeyboardUIMode -int 3

  # Enable subpixel font rendering on non-Apple LCDs
  defaults write NSGlobalDomain AppleFontSmoothing -int 2

  # Disable press-and-hold for keys in favor of key repeat
  defaults write NSGlobalDomain ApplePressAndHoldEnabled -bool false


  # Disable auto-correct
  defaults write NSGlobalDomain NSAutomaticSpellingCorrectionEnabled -bool false

  # Enable tap to click (Trackpad) for this user and for the login screen
  defaults write com.apple.driver.AppleBluetoothMultitouch.trackpad Clicking -bool true
  defaults -currentHost write NSGlobalDomain com.apple.mouse.tapBehavior -int 1
  defaults write NSGlobalDomain com.apple.mouse.tapBehavior -int 1

  # Tracking Speed
  # 0: Slow
  # 3: Fast
  defaults write NSGlobalDomain com.apple.trackpad.scaling -float 3

  # Allow quitting Finder via ⌘ + Q; doing so will also hide desktop icons
  defaults write com.apple.finder QuitMenuItem -bool true

  # Disable window animations and Get Info animations in Finder
  defaults write com.apple.finder DisableAllAnimations -bool true

  # Show all filename extensions in Finder
  defaults write NSGlobalDomain AppleShowAllExtensions -bool true

  # Show hidden files
  defaults write com.apple.finder AppleShowAllFiles -string "YES"

  # Show status bar in Finder
  defaults write com.apple.finder ShowStatusBar -bool true

  # Allow text selection in Quick Look
  defaults write com.apple.finder QLEnableTextSelection -bool true

  # Disable disk image verification
  defaults write com.apple.frameworks.diskimages skip-verify -bool true
  defaults write com.apple.frameworks.diskimages skip-verify-locked -bool true
  defaults write com.apple.frameworks.diskimages skip-verify-remote -bool true

  # Automatically open a new Finder window when a volume is mounted
  defaults write com.apple.frameworks.diskimages auto-open-ro-root -bool true
  defaults write com.apple.frameworks.diskimages auto-open-rw-root -bool true
  defaults write com.apple.finder OpenWindowForNewRemovableDisk -bool true

  # Display full POSIX path as Finder window title
  defaults write com.apple.finder _FXShowPosixPathInTitle -bool true

  # Avoid creating .DS_Store files on network volumes
  defaults write com.apple.desktopservices DSDontWriteNetworkStores -bool true

  # Disable the warning when changing a file extension
  defaults write com.apple.finder FXEnableExtensionChangeWarning -bool false

  # Show item info below desktop icons
  /usr/libexec/PlistBuddy -c "Set :DesktopViewSettings:IconViewSettings:showItemInfo true" ~/Library/Preferences/com.apple.finder.plist

  # Enable snap-to-grid for desktop icons
  /usr/libexec/PlistBuddy -c "Set :DesktopViewSettings:IconViewSettings:arrangeBy grid" ~/Library/Preferences/com.apple.finder.plist

  # Disable the warning before emptying the Trash
  defaults write com.apple.finder WarnOnEmptyTrash -bool false

  # Enable AirDrop over Ethernet and on unsupported Macs running Lion
  defaults write com.apple.NetworkBrowser BrowseAllInterfaces -bool true

  # Show the ~/Library folder
  chflags nohidden ~/Library

  # Enable highlight hover effect for the grid view of a stack (Dock)
  defaults write com.apple.dock mouse-over-hilte-stack -bool true

  # Enable spring loading for all Dock items
  defaults write com.apple.dock enable-spring-load-actions-on-all-items -bool true

  # Show indicator lights for open applications in the Dock
  defaults write com.apple.dock show-process-indicators -bool true

  # Don’t animate opening applications from the Dock
  defaults write com.apple.dock launchanim -bool false

  # Remove the auto-hiding Dock delay
  defaults write com.apple.Dock autohide-delay -float 0

  # Remove the animation when hiding/showing the Dock (actually, make it fast. If you want to remove, use 0)
  defaults write com.apple.dock autohide-time-modifier -float 0.25

  # Enable the 2D Dock
  defaults write com.apple.dock no-glass -bool true

  # Automatically hide and show the Dock
  defaults write com.apple.dock autohide -bool true

  # Move Dock to the left (can be 'left', 'right', 'bottom')
  defaults write com.apple.dock 'orientation' -string 'left'

  # Enable Safari’s debug menu
  defaults write com.apple.Safari IncludeInternalDebugMenu -bool true

  # Make Safari’s search banners default to Contains instead of Starts With
  defaults write com.apple.Safari FindOnPageMatchesWordStartsOnly -bool false

  # Remove useless icons from Safari’s bookmarks bar
  defaults write com.apple.Safari ProxiesInBookmarksBar "()"

  # Add a context menu item for showing the Web Inspector in web views
  defaults write NSGlobalDomain WebKitDeveloperExtras -bool true

  # Enable the debug menu in Address Book
  defaults write com.apple.addressbook ABShowDebugMenu -bool true

  # Enable the debug menu in iCal
  defaults write com.apple.iCal IncludeDebugMenu -bool true

  # Only use UTF-8 in Terminal.app
  defaults write com.apple.terminal StringEncodings -array 4

  # Copy email addresses as `foo@example.com` instead of `Foo Bar <foo@example.com>` in Mail.app
  defaults write com.apple.mail AddressesIncludeNameOnPasteboard -bool false

  # Enable Dashboard dev mode (allows keeping widgets on the desktop)
  defaults write com.apple.dashboard devmode -bool true

  # Reset Launchpad
  [ -e ~/Library/Application\ Support/Dock/*.db ] && rm ~/Library/Application\ Support/Dock/*.db

  # Prevent Time Machine from prompting to use new hard drives as backup volume
  defaults write com.apple.TimeMachine DoNotOfferNewDisksForBackup -bool true

  # Allow text expansion
  defaults write -g WebAutomaticTextReplacementEnabled -bool true

  # Allow AAC codec for Bluetooth headphones
  defaults write bluetoothaudiod "Enable AAC codec" -bool true

  # Remove all desktop icons
  defaults write com.apple.finder CreateDesktop -bool false

  # opening and closing windows and popovers
  defaults write -g NSAutomaticWindowAnimationsEnabled -bool false

  # smooth scrolling
  defaults write -g NSScrollAnimationEnabled -bool false

  # showing and hiding sheets, resizing preference windows, zooming windows
  # float 0 doesn't work
  defaults write -g NSWindowResizeTime -float 0.001

  # opening and closing Quick Look windows
  defaults write -g QLPanelAnimationDuration -float 0

  # rubberband scrolling (doesn't affect web views)
  defaults write -g NSScrollViewRubberbanding -bool false

  # resizing windows before and after showing the version browser
  # also disabled by NSWindowResizeTime -float 0.001
  defaults write -g NSDocumentRevisionsWindowTransformAnimation -bool false

  # showing a toolbar or menu bar in full screen
  defaults write -g NSToolbarFullScreenAnimationDuration -float 0

  # scrolling column views
  defaults write -g NSBrowserColumnAnimationSpeedMultiplier -float 0

  # showing the Dock
  defaults write com.apple.dock autohide-time-modifier -float 0
  defaults write com.apple.dock autohide-delay -float 0

  # showing and hiding Mission Control, command+numbers
  defaults write com.apple.dock expose-animation-duration -float 0

  # showing and hiding Launchpad
  defaults write com.apple.dock springboard-show-duration -float 0

  # dialog boxes
  defaults write NSGlobalDomain NSWindowResizeTime .001

  # Enable font smoothing
  defaults write -g CGFontRenderingFontSmoothingDisabled -bool NO
  defaults -currentHost write -globalDomain AppleFontSmoothing -int 2

  # Kill affected applications
  for app in Finder Dock Mail Safari iTunes iCal Address\ Book SystemUIServer; do killall "$app" > /dev/null 2>&1; done
  echo "macOS Hacks Done. Note that some of these changes require a logout/restart to take effect."
}

