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

# Prompt for keyboard layout if config doesn't exist
if [ ! -f /etc/hypr-greeter/config.json ]; then
    echo ""
    echo "Select your keyboard layout:"
    echo "  1) us - US English"
    echo "  2) de - German"
    echo "  3) fr - French"
    echo "  4) es - Spanish"
    echo "  5) it - Italian"
    echo "  6) pt - Portuguese"
    echo "  7) ru - Russian"
    echo "  8) uk - UK English"
    echo "  9) ch - Swiss German"
    echo " 10) be - Belgian"
    echo " 11) ca - Canadian"
    echo " 12) dk - Danish"
    echo " 13) fi - Finnish"
    echo " 14) no - Norwegian"
    echo " 15) se - Swedish"
    echo " 16) pl - Polish"
    echo " 17) cz - Czech"
    echo " 18) hu - Hungarian"
    echo " 19) ro - Romanian"
    echo " 20) tr - Turkish"
    echo " 21) jp - Japanese"
    echo " 22) kr - Korean"
    echo " 23) cn - Chinese"
    echo " 24) br - Brazilian Portuguese"
    echo " 25) Skip (no keyboard layout configuration)"
    echo ""
    read -r -p "Enter your choice (1-25) [1]: " layout_choice
    layout_choice=${layout_choice:-1}
    
    case $layout_choice in
        1) KEYBOARD_LAYOUT="us" ;;
        2) KEYBOARD_LAYOUT="de" ;;
        3) KEYBOARD_LAYOUT="fr" ;;
        4) KEYBOARD_LAYOUT="es" ;;
        5) KEYBOARD_LAYOUT="it" ;;
        6) KEYBOARD_LAYOUT="pt" ;;
        7) KEYBOARD_LAYOUT="ru" ;;
        8) KEYBOARD_LAYOUT="uk" ;;
        9) KEYBOARD_LAYOUT="ch" ;;
        10) KEYBOARD_LAYOUT="be" ;;
        11) KEYBOARD_LAYOUT="ca" ;;
        12) KEYBOARD_LAYOUT="dk" ;;
        13) KEYBOARD_LAYOUT="fi" ;;
        14) KEYBOARD_LAYOUT="no" ;;
        15) KEYBOARD_LAYOUT="se" ;;
        16) KEYBOARD_LAYOUT="pl" ;;
        17) KEYBOARD_LAYOUT="cz" ;;
        18) KEYBOARD_LAYOUT="hu" ;;
        19) KEYBOARD_LAYOUT="ro" ;;
        20) KEYBOARD_LAYOUT="tr" ;;
        21) KEYBOARD_LAYOUT="jp" ;;
        22) KEYBOARD_LAYOUT="kr" ;;
        23) KEYBOARD_LAYOUT="cn" ;;
        24) KEYBOARD_LAYOUT="br" ;;
        25) KEYBOARD_LAYOUT="" ;;
        *) KEYBOARD_LAYOUT="us" ;;
    esac
    
    # Create config with selected keyboard layout
    if [ -n "$KEYBOARD_LAYOUT" ]; then
        echo "Setting keyboard layout to: $KEYBOARD_LAYOUT"
        sed "s/\"keyboard_layout\": \"us\"/\"keyboard_layout\": \"$KEYBOARD_LAYOUT\"/" config.json > /etc/hypr-greeter/config.json
    else
        echo "Skipping keyboard layout configuration"
        # Remove keyboard_layout field if user skipped
        grep -v '"keyboard_layout"' config.json > /etc/hypr-greeter/config.json
    fi
else
    echo "Config already exists at /etc/hypr-greeter/config.json - skipping"
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