# Paperclip Todo Backup to Nextcloud

This directory contains scripts to backup your Paperclip todos to Nextcloud using WebDAV.

## Setup

### 1. Configure Nextcloud Connection

Run the setup command to configure your Nextcloud credentials:

```bash
./backup-to-nextcloud.sh setup
```

You'll be prompted to enter:
- **Nextcloud URL**: Your Nextcloud server URL (e.g., `https://cloud.example.com`)
- **Username**: Your Nextcloud username
- **Password**: Your Nextcloud password or app password (recommended)
- **Backup folder**: Folder name in Nextcloud to store backups (default: `paperclip`)

> **Security Note**: For better security, use an App Password instead of your main password. You can create one in Nextcloud Settings → Security → Devices & sessions → Create new app password.

### 2. Test the Backup

Once configured, test the backup:

```bash
./backup-to-nextcloud.sh backup
```

This will:
- Upload your current `todos.json` with a timestamp
- Create/update a `todos_latest.json` file for easy access

## Usage

### Manual Backup Commands

```bash
# Backup todos to Nextcloud
./backup-to-nextcloud.sh backup

# Check backup status
./backup-to-nextcloud.sh status

# List all available backups
./backup-to-nextcloud.sh list

# Restore from a specific backup
./backup-to-nextcloud.sh restore

# Show help
./backup-to-nextcloud.sh help
```

### Automatic Backups

Set up automatic backups using cron:

```bash
# Edit your crontab
crontab -e

# Add one of these lines for automatic backups:

# Backup every hour
0 * * * * /home/seawn/tokyo-todo/scripts/auto-backup.sh

# Backup every 6 hours
0 */6 * * * /home/seawn/tokyo-todo/scripts/auto-backup.sh

# Backup daily at 2 AM
0 2 * * * /home/seawn/tokyo-todo/scripts/auto-backup.sh

# Backup every time you log in (add to your shell profile)
echo "/home/seawn/tokyo-todo/scripts/auto-backup.sh" >> ~/.bashrc
```

## File Structure

```
scripts/
├── backup-to-nextcloud.sh    # Main backup script
├── auto-backup.sh            # Automatic backup wrapper for cron
├── .nextcloud-config         # Configuration file (created after setup)
├── backup.log               # Log file for automatic backups
└── README.md                # This file
```

## Backup Files in Nextcloud

Your backups will be stored in Nextcloud at:
```
/paperclip/
├── todos_latest.json                    # Always the most recent backup
├── todos_backup_2025-08-30_14-30-15.json   # Timestamped backups
├── todos_backup_2025-08-30_15-45-22.json
└── ...
```

## Troubleshooting

### Connection Issues
- Verify your Nextcloud URL (include `https://`)
- Check if WebDAV is enabled on your Nextcloud server
- Use an App Password instead of your main password
- Ensure your Nextcloud account has sufficient storage space

### Permission Issues
- Make sure the scripts are executable: `chmod +x *.sh`
- Check that the `.nextcloud-config` file has proper permissions (600)

### Backup Failures
- Check the `backup.log` file for error details
- Verify your internet connection
- Ensure the backup directory exists in Nextcloud

## Security

- Configuration file (`.nextcloud-config`) has restricted permissions (600)
- Use App Passwords instead of main account passwords
- The config file is excluded from git (add to `.gitignore`)

## Integration Ideas

You could integrate this backup system into the Paperclip app itself by:
1. Adding a backup command to the app (e.g., `Ctrl+B`)
2. Automatic backup on app exit
3. Backup status indicator in the UI
4. Background sync functionality

---

**Note**: This backup system uses Nextcloud's WebDAV API, which is supported by most Nextcloud instances. If your Nextcloud server has WebDAV disabled, you'll need to enable it or use an alternative backup method.
