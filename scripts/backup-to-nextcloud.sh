#!/bin/bash

# Paperclip Todo Backup to Nextcloud
# This script backs up your todos.json file to Nextcloud using WebDAV

set -e

# Configuration (you'll need to set these)
NEXTCLOUD_URL=""           # e.g., "https://cloud.example.com"
NEXTCLOUD_USERNAME=""      # Your Nextcloud username
NEXTCLOUD_PASSWORD=""      # Your Nextcloud password or app password
BACKUP_PATH="paperclip"    # Path in Nextcloud where backups will be stored

# Local paths
TODO_FILE="$HOME/.local/share/paperclip/todos.json"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="$SCRIPT_DIR/.nextcloud-config"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  setup     - Configure Nextcloud connection"
    echo "  backup    - Backup todos to Nextcloud"
    echo "  restore   - Restore todos from Nextcloud"
    echo "  status    - Check backup status"
    echo "  list      - List available backups"
    echo ""
    echo "If no command is specified, 'backup' is used."
}

load_config() {
    if [[ -f "$CONFIG_FILE" ]]; then
        source "$CONFIG_FILE"
    else
        echo -e "${YELLOW}Warning: No configuration found. Run '$0 setup' first.${NC}"
        return 1
    fi
}

setup_config() {
    echo -e "${BLUE}Setting up Nextcloud backup configuration...${NC}"
    echo ""
    
    read -p "Nextcloud URL (e.g., https://cloud.example.com): " url
    read -p "Username: " username
    echo -n "Password (or app password): "
    read -s password
    echo ""
    read -p "Backup folder in Nextcloud (default: paperclip): " backup_path
    
    # Set defaults
    backup_path=${backup_path:-paperclip}
    
    # Test connection
    echo -e "${BLUE}Testing connection...${NC}"
    if curl -f -s -u "$username:$password" "$url/remote.php/dav/files/$username/" > /dev/null; then
        echo -e "${GREEN}✓ Connection successful!${NC}"
        
        # Save configuration
        cat > "$CONFIG_FILE" << EOF
NEXTCLOUD_URL="$url"
NEXTCLOUD_USERNAME="$username"
NEXTCLOUD_PASSWORD="$password"
BACKUP_PATH="$backup_path"
EOF
        
        chmod 600 "$CONFIG_FILE"  # Secure the config file
        echo -e "${GREEN}✓ Configuration saved to $CONFIG_FILE${NC}"
        
        # Create backup directory in Nextcloud
        create_backup_dir
        
    else
        echo -e "${RED}✗ Connection failed. Please check your credentials and URL.${NC}"
        return 1
    fi
}

create_backup_dir() {
    echo -e "${BLUE}Creating backup directory in Nextcloud...${NC}"
    
    # Create the backup directory (ignore error if it already exists)
    curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
         -X MKCOL \
         "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/" \
         2>/dev/null || true
    
    echo -e "${GREEN}✓ Backup directory ready: /$BACKUP_PATH/${NC}"
}

backup_todos() {
    if [[ ! -f "$TODO_FILE" ]]; then
        echo -e "${YELLOW}Warning: No todos.json file found at $TODO_FILE${NC}"
        return 1
    fi
    
    load_config || return 1
    
    # Generate backup filename with timestamp
    timestamp=$(date +"%Y-%m-%d_%H-%M-%S")
    backup_filename="todos_backup_$timestamp.json"
    
    echo -e "${BLUE}Backing up todos to Nextcloud...${NC}"
    echo "Local file: $TODO_FILE"
    echo "Remote path: /$BACKUP_PATH/$backup_filename"
    
    # Upload the file
    if curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
            -T "$TODO_FILE" \
            "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/$backup_filename"; then
        
        echo -e "${GREEN}✓ Backup successful!${NC}"
        echo "Backup saved as: $backup_filename"
        
        # Also create/update a "latest" backup
        if curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
                -T "$TODO_FILE" \
                "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/todos_latest.json"; then
            echo -e "${GREEN}✓ Latest backup updated${NC}"
        fi
        
        return 0
    else
        echo -e "${RED}✗ Backup failed!${NC}"
        return 1
    fi
}

list_backups() {
    load_config || return 1
    
    echo -e "${BLUE}Listing available backups...${NC}"
    
    # List files in backup directory
    curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
         -X PROPFIND \
         "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/" \
         --data '<?xml version="1.0"?>
                 <d:propfind xmlns:d="DAV:">
                   <d:prop>
                     <d:displayname/>
                     <d:getlastmodified/>
                     <d:getcontentlength/>
                   </d:prop>
                 </d:propfind>' \
         --header "Content-Type: text/xml" | \
    grep -E "(displayname|getlastmodified)" | \
    sed 's/<[^>]*>//g' | \
    grep -E "(todos_|\.json)" || echo "No backups found"
}

restore_todos() {
    load_config || return 1
    
    echo -e "${BLUE}Available backups:${NC}"
    list_backups
    echo ""
    
    read -p "Enter backup filename to restore (or 'latest' for most recent): " backup_file
    
    if [[ "$backup_file" == "latest" ]]; then
        backup_file="todos_latest.json"
    fi
    
    # Backup current file before restoring
    if [[ -f "$TODO_FILE" ]]; then
        backup_current="$TODO_FILE.backup.$(date +%s)"
        cp "$TODO_FILE" "$backup_current"
        echo -e "${YELLOW}Current todos backed up to: $backup_current${NC}"
    fi
    
    echo -e "${BLUE}Restoring from: $backup_file${NC}"
    
    # Download and restore
    if curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
            -o "$TODO_FILE" \
            "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/$backup_file"; then
        
        echo -e "${GREEN}✓ Restore successful!${NC}"
        echo "Todos restored from: $backup_file"
        return 0
    else
        echo -e "${RED}✗ Restore failed!${NC}"
        
        # Restore original file if backup exists
        if [[ -f "$backup_current" ]]; then
            mv "$backup_current" "$TODO_FILE"
            echo -e "${YELLOW}Original file restored${NC}"
        fi
        return 1
    fi
}

check_status() {
    load_config || return 1
    
    echo -e "${BLUE}Backup Status:${NC}"
    echo ""
    
    if [[ -f "$TODO_FILE" ]]; then
        local_size=$(stat -f%z "$TODO_FILE" 2>/dev/null || stat -c%s "$TODO_FILE" 2>/dev/null)
        local_modified=$(stat -f%m "$TODO_FILE" 2>/dev/null || stat -c%Y "$TODO_FILE" 2>/dev/null)
        local_date=$(date -r "$local_modified" "+%Y-%m-%d %H:%M:%S" 2>/dev/null || date -d "@$local_modified" "+%Y-%m-%d %H:%M:%S")
        
        echo "Local file:"
        echo "  Path: $TODO_FILE"
        echo "  Size: $local_size bytes"
        echo "  Modified: $local_date"
        echo ""
    else
        echo -e "${YELLOW}No local todos.json file found${NC}"
        echo ""
    fi
    
    # Check if latest backup exists
    if curl -f -s -u "$NEXTCLOUD_USERNAME:$NEXTCLOUD_PASSWORD" \
            -I "$NEXTCLOUD_URL/remote.php/dav/files/$NEXTCLOUD_USERNAME/$BACKUP_PATH/todos_latest.json" \
            > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Latest backup exists in Nextcloud${NC}"
    else
        echo -e "${YELLOW}No latest backup found in Nextcloud${NC}"
    fi
    
    echo ""
    echo "Run '$0 list' to see all available backups"
}

# Main script logic
case "${1:-backup}" in
    "setup")
        setup_config
        ;;
    "backup")
        backup_todos
        ;;
    "restore")
        restore_todos
        ;;
    "list")
        list_backups
        ;;
    "status")
        check_status
        ;;
    "help"|"-h"|"--help")
        print_usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        print_usage
        exit 1
        ;;
esac
