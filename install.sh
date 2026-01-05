#!/bin/bash
# Installation script for hypr-greeter

set -e

echo "Installing hypr-greeter..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Detect package manager
if command -v pacman &> /dev/null; then
    PKG_MANAGER="pacman"
    INSTALL_CMD="pacman -S --needed --noconfirm"
elif command -v apt &> /dev/null; then
    PKG_MANAGER="apt"
    INSTALL_CMD="apt install -y"
elif command -v dnf &> /dev/null; then
    PKG_MANAGER="dnf"
    INSTALL_CMD="dnf install -y"
elif command -v zypper &> /dev/null; then
    PKG_MANAGER="zypper"
    INSTALL_CMD="zypper install -y"
else
    echo "Unsupported package manager. Please install dependencies manually:"
    echo "  - greetd"
    echo "  - cage"
    echo "  - rust/cargo"
    exit 1
fi

echo "Detected package manager: $PKG_MANAGER"

# Install dependencies based on package manager
echo "Installing dependencies..."
case $PKG_MANAGER in
    pacman)
        $INSTALL_CMD greetd cage rust
        ;;
    apt)
        $INSTALL_CMD greetd cage cargo
        ;;
    dnf)
        $INSTALL_CMD greetd cage cargo
        ;;
    zypper)
        $INSTALL_CMD greetd cage cargo
        ;;
esac

# Ensure cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust/Cargo manually."
    exit 1
fi

# Build the greeter
echo "Building hypr-greeter..."
cargo build --release

# Install binary
echo "Installing binary..."
install -Dm755 target/release/hypr-greeter /usr/local/bin/hypr-greeter

# Create config directory and install default config
echo "Installing config..."
mkdir -p /etc/hypr-greeter
if [ ! -f /etc/hypr-greeter/config.json ]; then
    cp config.json /etc/hypr-greeter/config.json
fi

# Create greeter user if it doesn't exist
if ! id "greeter" &>/dev/null; then
    echo "Creating greeter user..."
    useradd -M -G video greeter
    passwd -d greeter
fi

# Create greetd data directory
mkdir -p /var/lib/greetd
chown -R greeter:greeter /var/lib/greetd

# Install greetd config
echo "Configuring greetd..."
mkdir -p /etc/greetd
cat > /etc/greetd/config.toml << 'EOF'
[terminal]
vt = 1

[default_session]
command = "cage -s -- /usr/local/bin/hypr-greeter"
user = "greeter"
EOF

# Enable greetd service
echo "Enabling greetd service..."
systemctl daemon-reload
systemctl enable greetd.service

echo ""
echo "Installation complete!"
echo ""
echo "Configuration file: /etc/hypr-greeter/config.json"
echo ""
echo "To start the greeter now:"
echo "  sudo systemctl start greetd"
echo ""
echo "To check logs:"
echo "  journalctl -u greetd -f"
echo ""