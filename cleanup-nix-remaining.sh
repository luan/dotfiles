#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting nix-darwin remaining cleanup script...${NC}"

# Remove nix daemon service files
echo -e "${BLUE}Removing nix daemon service files...${NC}"
sudo rm -f /Library/LaunchDaemons/org.nixos.nix-daemon.plist
sudo rm -f /Library/LaunchDaemons/org.nixos.darwin-store.plist

# Remove any remaining nix-darwin files
echo -e "${BLUE}Removing any remaining nix-darwin files...${NC}"
sudo rm -rf /etc/nix
sudo rm -rf /etc/static
sudo rm -f /etc/bashrc.backup-before-nix
sudo rm -f /etc/zshrc.backup-before-nix

# Remove nix-related user files
echo -e "${BLUE}Removing nix-related user files...${NC}"
sudo rm -rf /var/root/.nix-profile /var/root/.nix-defexpr /var/root/.nix-channels
rm -rf ~/.nix-profile ~/.nix-defexpr ~/.nix-channels

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

echo -e "${GREEN}Cleanup of remaining Nix files complete!${NC}"
echo -e "${YELLOW}Note: The /nix directory still exists but is empty and cannot be removed due to System Integrity Protection.${NC}"
echo -e "${YELLOW}You may need to disable SIP and reboot to fully remove it, or you can safely ignore it as it's now empty.${NC}" 