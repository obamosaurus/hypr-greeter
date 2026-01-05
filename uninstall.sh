#!/bin/bash
# Uninstall script for hypr-greeter

set -e

echo "Uninstalling hypr-greeter..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Stop and disable greetd service
echo "Stopping greetd service..."
systemctl stop greetd.service || true
systemctl disable greetd.service || true

# Remove binary
echo "Removing binary..."
rm -f /usr/local/bin/hypr-greeter

# Ask about config removal
read -p "Remove configuration files? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing config files..."
    rm -rf /etc/hypr-greeter
    rm -f /etc/greetd/config.toml
    rm -rf /var/lib/greetd
fi

echo ""
echo "Uninstallation complete!"
echo ""