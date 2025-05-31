#!/bin/bash

SERVICE_FILE="/etc/systemd/system/matcha_watcher.service"

if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root. Please use sudo."
    exit 1
fi

cat <<EOF > "$SERVICE_FILE"
[Unit]
Description=matcha_watcher Service
After=network.target

[Service]
Type=simple
User=alistair
WorkingDirectory=/home/alistair/matcha_watcher
ExecStart=/home/alistair/matcha_watcher/target/release/matcha_watcher
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload

systemctl enable matcha_watcher

systemctl start matcha_watcher

echo "Matcha Watcher service installed and started successfully."