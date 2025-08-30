#!/bin/bash

# Auto-backup script for Paperclip todos
# This script can be run via cron for automatic backups

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_SCRIPT="$SCRIPT_DIR/backup-to-nextcloud.sh"
LOG_FILE="$SCRIPT_DIR/backup.log"

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Check if backup script exists
if [[ ! -f "$BACKUP_SCRIPT" ]]; then
    log "ERROR: Backup script not found at $BACKUP_SCRIPT"
    exit 1
fi

# Run backup and log result
log "Starting automatic backup..."

if "$BACKUP_SCRIPT" backup >> "$LOG_FILE" 2>&1; then
    log "SUCCESS: Backup completed successfully"
    exit 0
else
    log "ERROR: Backup failed"
    exit 1
fi
