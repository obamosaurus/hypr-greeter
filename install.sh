#!/bin/bash
# Installation script for hypr-greeter
# Run with: chmod +x install.sh && sudo ./install.sh

set -e

echo "Installing hypr-greeter..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Build the greeter
echo "Building hypr-greeter..."
cargo build --release

# Install binary
echo "Installing binary..."
install -Dm755 target/release/hypr-greeter /usr/local/bin/hypr-greeter

# Create config directory
echo "Creating config directory..."
mkdir -p /etc/hypr-greeter

# Install default config if it doesn't exist
if [ ! -f /etc/hypr-greeter/config.json ]; then
    echo "Installing default config..."
    cat > /etc/hypr-greeter/config.json << 'EOF'
{
  "last_user": "",
  "sessions": [
    {
      "name": "Hyprland",
      "command": "Hyprland"
    },
    {
      "name": "Sway",
      "command": "sway"
    },
    {
      "name": "TTY",
      "command": "/bin/bash"
    }
  ],
  "ui": {
    "show_clock": true,
    "clock_format": "%H:%M",
    "show_date": true,
    "date_format": "%A, %d %B %Y",
    "colors": {
      "background": "#1a1b26",
      "foreground": "#c0caf5",
      "focused": "#f7768e",
      "error": "#f7768e",
      "success": "#9ece6a"
    }
  },
  "security": {
    "clear_password_on_error": true,
    "mask_password": true,
    "input_timeout": 0
  }
}
EOF
fi

# Create greeter user if it doesn't exist
if ! id "greeter" &>/dev/null; then
    echo "Creating greeter user..."
    useradd -M -G video greeter
    passwd -d greeter
fi

# Install greetd config
echo "Installing greetd config..."
mkdir -p /etc/greetd
cat > /etc/greetd/config.toml << 'EOF'
[terminal]
vt = 1

[default_session]
command = "cage -s -- alacritty -e /usr/local/bin/hypr-greeter"
user = "greeter"

[initial_session]
command = "cage -s -- alacritty -e /usr/local/bin/hypr-greeter"
user = "greeter"
EOF

# Create systemd override directory
mkdir -p /etc/systemd/system/greetd.service.d

# Install systemd override
cat > /etc/systemd/system/greetd.service.d/override.conf << 'EOF'
[Service]
After=systemd-user-sessions.service plymouth-quit-wait.service
Environment="XDG_SESSION_TYPE=wayland"
TimeoutStartSec=30s
Restart=always
RestartSec=1s
KillMode=process
EOF

# Set permissions
chmod 644 /etc/hypr-greeter/config.json
chmod 644 /etc/greetd/config.toml
chmod 755 /usr/local/bin/hypr-greeter

# Enable and start greetd
echo "Enabling greetd service..."
systemctl daemon-reload
systemctl enable greetd.service

echo ""
echo "Installation complete!"
echo ""
echo "Required packages (install with pacman):"
echo "  - greetd"
echo "  - cage"
echo "  - alacritty (or another terminal emulator)"
echo ""
echo "To start the greeter now:"
echo "  sudo systemctl start greetd"
echo ""
echo "To check logs:"
echo "  journalctl -u greetd -f"
echo ""