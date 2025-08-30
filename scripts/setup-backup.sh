#!/bin/bash

# Quick setup guide for Paperclip Nextcloud backup

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_SCRIPT="$SCRIPT_DIR/backup-to-nextcloud.sh"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Paperclip Nextcloud Backup Setup     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${YELLOW}This will help you set up automatic backups of your todos to Nextcloud.${NC}"
echo ""

echo -e "${BLUE}📋 What you'll need:${NC}"
echo "• Your Nextcloud server URL (e.g., https://cloud.example.com)"
echo "• Your Nextcloud username"
echo "• Your Nextcloud password (or preferably an App Password)"
echo ""

echo -e "${BLUE}🔐 For better security, create an App Password:${NC}"
echo "1. Log into your Nextcloud web interface"
echo "2. Go to Settings → Security → Devices & sessions"
echo "3. Create a new app password for 'Paperclip Backup'"
echo "4. Use that password instead of your main password"
echo ""

read -p "Ready to configure? (y/N): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${BLUE}Running configuration...${NC}"
    echo ""
    
    "$BACKUP_SCRIPT" setup
    
    if [[ $? -eq 0 ]]; then
        echo ""
        echo -e "${GREEN}🎉 Setup complete!${NC}"
        echo ""
        echo -e "${BLUE}Next steps:${NC}"
        echo "1. Test backup: $BACKUP_SCRIPT backup"
        echo "2. Check status: $BACKUP_SCRIPT status"
        echo "3. Set up automatic backups (see README.md)"
        echo ""
        echo -e "${YELLOW}💡 Tip: Add this to your shell profile for backups on login:${NC}"
        echo "echo '$SCRIPT_DIR/auto-backup.sh' >> ~/.bashrc"
    fi
else
    echo ""
    echo -e "${YELLOW}Setup cancelled. Run this script again when ready.${NC}"
    echo ""
    echo -e "${BLUE}Manual setup:${NC}"
    echo "$BACKUP_SCRIPT setup"
fi

echo ""
echo -e "${BLUE}📖 For more information, see: scripts/README.md${NC}"
