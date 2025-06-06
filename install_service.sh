#!/bin/bash

SERVICE_FILE="/etc/systemd/system/matcha_watcher.service"
APP_USER="alistair"
WORKING_DIR="/home/$APP_USER/matcha_watcher"
HELPER_SCRIPT_NAME="run_latest_matcha_watcher.sh"
HELPER_SCRIPT_PATH="$WORKING_DIR/$HELPER_SCRIPT_NAME"
BINARY_NAME="matcha_watcher"
GITHUB_REPO="AlistairKeiller/matcha_watcher"

if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root. Please use sudo."
    exit 1
fi

if ! command -v jq &> /dev/null || ! command -v curl &> /dev/null; then
    echo "Error: jq and curl are required for downloading the latest release."
    echo "Please install them first (e.g., sudo apt update && sudo apt install -y jq curl)."
    exit 1
fi

read -p "Enter your Discord token: " DISCORD_TOKEN

if ! id "$APP_USER" &>/dev/null; then
    echo "Error: User '$APP_USER' does not exist. Please create the user or change APP_USER in the script."
    exit 1
fi
mkdir -p "$WORKING_DIR"
chown "$APP_USER:$APP_USER" "$WORKING_DIR"

cat <<EOF_HELPER > "$HELPER_SCRIPT_PATH"
#!/bin/bash

set -e

REPO="$GITHUB_REPO"
DOWNLOAD_DIR="$WORKING_DIR"
TARGET_BINARY_NAME="$BINARY_NAME"

echo "Matcha Watcher Helper: Starting update process..."
echo "Matcha Watcher Helper: Repository: \$REPO"
echo "Matcha Watcher Helper: Target binary: \$TARGET_BINARY_NAME, Download directory: \$DOWNLOAD_DIR"

echo "Matcha Watcher Helper: Fetching latest release information from GitHub API..."
LATEST_RELEASE_INFO=\$(curl -s -H "Accept: application/vnd.github.v3+json" -H "User-Agent: $GITHUB_REPO-systemd-installer/1.0" "https://api.github.com/repos/\$REPO/releases/latest")

if echo "\$LATEST_RELEASE_INFO" | jq -e '.message' > /dev/null && ! echo "\$LATEST_RELEASE_INFO" | jq -e '.assets' > /dev/null ; then
    echo "Matcha Watcher Helper: Error fetching release info. API Response: \$LATEST_RELEASE_INFO"
    exit 1
fi

DOWNLOAD_URL=\$(echo "\$LATEST_RELEASE_INFO" | jq -r ".assets[] | select(.name == \\"\$TARGET_BINARY_NAME\\") | .browser_download_url")

if [ -z "\$DOWNLOAD_URL" ] || [ "\$DOWNLOAD_URL" == "null" ]; then
    echo "Matcha Watcher Helper: Error: Could not find download URL for '\$TARGET_BINARY_NAME' in the latest release."
    echo "Matcha Watcher Helper: Ensure an asset named '\$TARGET_BINARY_NAME' exists in the latest release of \$REPO."
    echo "Matcha Watcher Helper: Available assets:"
    echo "\$LATEST_RELEASE_INFO" | jq -r '.assets[] | .name'
    exit 1
fi

echo "Matcha Watcher Helper: Downloading '\$TARGET_BINARY_NAME' from \$DOWNLOAD_URL..."
TEMP_DOWNLOAD_PATH="\$DOWNLOAD_DIR/\${TARGET_BINARY_NAME}.tmp.\$\$"
curl -L -f -o "\$TEMP_DOWNLOAD_PATH" "\$DOWNLOAD_URL"

if [ ! -s "\$TEMP_DOWNLOAD_PATH" ]; then
    echo "Matcha Watcher Helper: Error: Download failed or resulted in an empty file."
    rm -f "\$TEMP_DOWNLOAD_PATH"
    exit 1
fi

mv "\$TEMP_DOWNLOAD_PATH" "\$DOWNLOAD_DIR/\$TARGET_BINARY_NAME"
echo "Matcha Watcher Helper: Download complete: \$DOWNLOAD_DIR/\$TARGET_BINARY_NAME"

echo "Matcha Watcher Helper: Setting execute permissions..."
chmod +x "\$DOWNLOAD_DIR/\$TARGET_BINARY_NAME"

echo "Matcha Watcher Helper: Starting \$TARGET_BINARY_NAME..."
exec "\$DOWNLOAD_DIR/\$TARGET_BINARY_NAME"
EOF_HELPER

chown "$APP_USER:$APP_USER" "$HELPER_SCRIPT_PATH"
chmod +x "$HELPER_SCRIPT_PATH"
echo "Helper script created at $HELPER_SCRIPT_PATH"

cat <<EOF_SERVICE > "$SERVICE_FILE"
[Unit]
Description=matcha_watcher Service (auto-updating)
After=network.target network-online.target
Requires=network-online.target

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$WORKING_DIR
Environment="DISCORD_TOKEN=$DISCORD_TOKEN"
ExecStart=$HELPER_SCRIPT_PATH
Restart=on-failure
RestartSec=30s

[Install]
WantedBy=multi-user.target
EOF_SERVICE

echo "Systemd service file created at $SERVICE_FILE"

systemctl daemon-reload
systemctl enable matcha_watcher.service
systemctl restart matcha_watcher.service

echo "Matcha Watcher service (auto-updating) installed and (re)started successfully."
echo "The service will attempt to download the latest '$BINARY_NAME' from '$GITHUB_REPO' on each start."
echo "To check logs: journalctl -u matcha_watcher.service -f"