#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting nix-darwin cleanup script...${NC}"
echo -e "${YELLOW}WARNING: This script will remove nix-darwin from your system.${NC}"
echo -e "${YELLOW}Make sure you have already migrated to Homebrew Bundle.${NC}"
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}Cleanup aborted.${NC}"
    exit 1
fi

# Check if nix-darwin is installed
if [ -d "/etc/nix" ] || [ -d "/nix" ]; then
    echo -e "${BLUE}Removing nix-darwin...${NC}"
    
    # Uninstall nix-darwin if the uninstall script exists
    if [ -f "/run/current-system/sw/bin/darwin-rebuild" ]; then
        echo -e "${BLUE}Running nix-darwin uninstaller...${NC}"
        /run/current-system/sw/bin/darwin-rebuild --uninstall
    fi
    
    # Remove nix store and related directories
    echo -e "${BLUE}Removing nix store and related directories...${NC}"
    sudo rm -rf /nix /etc/nix /var/root/.nix-profile /var/root/.nix-defexpr /var/root/.nix-channels ~/.nix-profile ~/.nix-defexpr ~/.nix-channels
    
    # Remove nix daemon service files
    echo -e "${BLUE}Removing nix daemon service files...${NC}"
    sudo rm -f /Library/LaunchDaemons/org.nixos.nix-daemon.plist
    sudo rm -f /Library/LaunchDaemons/org.nixos.darwin-store.plist
    
    # Remove any remaining nix-darwin files
    echo -e "${BLUE}Removing any remaining nix-darwin files...${NC}"
    sudo rm -rf /etc/static
    sudo rm -f /etc/bashrc.backup-before-nix
    sudo rm -f /etc/zshrc.backup-before-nix
    
    echo -e "${GREEN}nix-darwin has been removed from your system.${NC}"
else
    echo -e "${YELLOW}nix-darwin does not appear to be installed.${NC}"
fi

# Remove nix-related entries from shell config files
echo -e "${BLUE}Removing nix-related entries from shell config files...${NC}"
for file in ~/.zshrc ~/.bashrc ~/.bash_profile; do
    if [ -f "$file" ]; then
        echo -e "${BLUE}Checking $file for nix-related entries...${NC}"
        # Create a backup
        cp "$file" "$file.bak"
        # Remove nix-related lines
        grep -v "nix" "$file.bak" > "$file" || true
    fi
done

echo -e "${GREEN}Cleanup complete!${NC}"
echo -e "${YELLOW}Note: You may need to restart your system for all changes to take effect.${NC}"
echo -e "${YELLOW}You should also check your shell configuration files manually to ensure all nix-related entries have been removed.${NC}" 