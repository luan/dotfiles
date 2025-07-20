#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting setup script...${NC}"

# Install Homebrew if not installed
if ! command -v brew &> /dev/null; then
    echo -e "${YELLOW}Homebrew not found. Installing Homebrew...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH for the current session
    if [[ $(uname -m) == "arm64" ]]; then
        echo -e "${YELLOW}Adding Homebrew to PATH for Apple Silicon...${NC}"
        eval "$(/opt/homebrew/bin/brew shellenv)"
    else
        echo -e "${YELLOW}Adding Homebrew to PATH for Intel Mac...${NC}"
        eval "$(/usr/local/bin/brew shellenv)"
    fi
else
    echo -e "${GREEN}Homebrew is already installed.${NC}"
fi

# Install packages from Brewfile
echo -e "${BLUE}Installing packages from Brewfile...${NC}"
brew bundle --file="$HOME/dotfiles/Brewfile"

# Create symlinks for dotfiles
echo -e "${BLUE}Setting up dotfiles...${NC}"
mkdir -p "$HOME/.config"

# Copy dotfiles
echo -e "${BLUE}Copying dotfiles...${NC}"
cp -f "$HOME/dotfiles/nix/home/dotfiles/dir_colors" "$HOME/.dir_colors"
cp -f "$HOME/dotfiles/nix/home/dotfiles/inputrc" "$HOME/.inputrc"

# Set macOS defaults
echo -e "${BLUE}Setting macOS defaults...${NC}"
source "$HOME/dotfiles/macos-defaults.sh"

echo -e "${GREEN}Setup complete!${NC}"
echo -e "${YELLOW}Note: Some changes may require a restart to take effect.${NC}" 