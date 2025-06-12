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

# Check and install dependencies
echo "Checking dependencies..."

# Function to check if a package is installed
is_installed() {
    pacman -Qi "$1" &> /dev/null
}

# Install required packages
DEPS=("greetd" "cage" "alacritty" "cargo" "rust")
for pkg in "${DEPS[@]}"; do
    if ! is_installed "$pkg"; then
        echo "Installing $pkg..."
        if ! pacman -S --noconfirm "$pkg"; then
            # If package not found in official repos, try AUR
            if command -v yay &> /dev/null; then
                echo "Package not found in official repos, trying AUR..."
                su - $SUDO_USER -c "yay -S --noconfirm $pkg"
            else
                echo "Error: $pkg not found in official repos and yay is not installed"
                echo "Please install $pkg manually"
                exit 1
            fi
        fi
    else
        echo "$pkg is already installed"
    fi
done

# Ensure cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed or not in PATH after attempted install."
    echo "Please ensure Rust and Cargo are installed and available in your PATH."
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
    cp "$(dirname "$0")/config.json" /etc/hypr-greeter/config.json
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

# Clean up any pacnew file if it exists
if [ -f /etc/greetd/config.toml.pacnew ]; then
    echo "Removing old .pacnew file..."
    rm /etc/greetd/config.toml.pacnew
fi

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
echo "To start the greeter now:"
echo "  sudo systemctl start greetd"
echo ""
echo "To check logs:"
echo "  journalctl -u greetd -f"
echo ""