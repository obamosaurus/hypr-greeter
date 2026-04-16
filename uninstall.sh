#!/bin/bash
# Uninstall script for hypr-greeter
set -e

echo "Uninstalling hypr-greeter..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (use sudo)"
    exit 1
fi

# Stop greetd service
echo "Stopping greetd service..."
systemctl stop greetd.service 2>/dev/null || true
systemctl disable greetd.service 2>/dev/null || true

# Remove binaries
echo "Removing binaries..."
rm -f /usr/local/bin/hypr-greeter
# Legacy: older versions installed a shell wrapper at this path. Keep the rm
# so upgraders don't leave it behind.
rm -f /usr/local/bin/hypr-greeter-wrapper

# Ask about config removal
read -p "Remove configuration files (/etc/hypr-greeter)? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing config files..."
    rm -rf /etc/hypr-greeter
    rm -rf /etc/systemd/system/greetd.service.d
fi

# Ask about greetd state
read -p "Remove greetd state (/var/lib/greetd)? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Removing greetd state..."
    rm -rf /var/lib/greetd
fi

# Ask about greeter user
if id "greeter" &>/dev/null; then
    read -p "Remove greeter user? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Removing greeter user..."
        userdel greeter 2>/dev/null || true
    fi
fi

echo ""
echo "Uninstallation complete!"
echo ""
echo "You may want to remove these packages if no longer needed:"
echo "  - hyprland"
echo "  - foot"
echo "  - greetd"
echo "  - rust"
echo ""
echo "To remove greetd config: rm -f /etc/greetd/config.toml"
echo ""
