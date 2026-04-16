#!/bin/bash
# Installation script for hypr-greeter
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Installing hypr-greeter..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root (use sudo)"
    exit 1
fi

# Check and install dependencies
echo "Checking dependencies..."

is_installed() {
    pacman -Qi "$1" &> /dev/null
}

install_pkg() {
    local pkg="$1"
    if ! is_installed "$pkg"; then
        echo "Installing $pkg..."
        if ! pacman -S --noconfirm "$pkg" 2>/dev/null; then
            if command -v yay &> /dev/null && [ -n "$SUDO_USER" ]; then
                echo "Package not found in official repos, trying AUR..."
                su - "$SUDO_USER" -c "yay -S --noconfirm $pkg"
            else
                echo "Error: $pkg not found and yay is not available"
                echo "Please install $pkg manually"
                exit 1
            fi
        fi
    else
        echo "$pkg is already installed"
    fi
}

disable_conflicting_display_managers() {
    local services=(
        "ly.service"
        "display-manager.service"
        "gdm.service"
        "sddm.service"
        "lightdm.service"
        "lxdm.service"
        "xdm.service"
    )

    echo "Disabling conflicting display manager services..."
    for service in "${services[@]}"; do
        if systemctl list-unit-files "$service" --no-legend 2>/dev/null | grep -q "^$service"; then
            if systemctl is-enabled "$service" &>/dev/null; then
                echo "Disabling $service..."
                systemctl disable "$service"
            fi

            if systemctl is-active "$service" &>/dev/null; then
                echo "Stopping $service..."
                systemctl stop "$service"
            fi
        fi
    done
}

DEPS=("greetd" "hyprland" "foot" "cargo" "rust")
for pkg in "${DEPS[@]}"; do
    install_pkg "$pkg"
done

# Ensure cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed or not in PATH."
    echo "Please ensure Rust and Cargo are installed and available in your PATH."
    exit 1
fi

# Build the greeter
echo "Building hypr-greeter..."
cd "$SCRIPT_DIR"
cargo build --release

# Install binaries
echo "Installing binaries..."
install -Dm755 "$SCRIPT_DIR/target/release/hypr-greeter" /usr/local/bin/hypr-greeter

# Create config directory and install default config
echo "Setting up configuration..."
mkdir -p /etc/hypr-greeter

if [ ! -f /etc/hypr-greeter/config.toml ]; then
    echo "Installing default config..."
    cp "$SCRIPT_DIR/config.toml" /etc/hypr-greeter/config.toml
    chmod 644 /etc/hypr-greeter/config.toml
else
    echo "Config already exists, skipping..."
fi

# Create greeter user if it doesn't exist
if ! id "greeter" &>/dev/null; then
    echo "Creating greeter user..."
    useradd -M -G video greeter
    passwd -d greeter
fi

# Ensure greeter user has a home directory
if id "greeter" &>/dev/null; then
    GREETER_HOME=$(getent passwd greeter | cut -d: -f6)
    if [ -z "$GREETER_HOME" ] || [ "$GREETER_HOME" = "/" ] || [ "$GREETER_HOME" = "/nonexistent" ]; then
        GREETER_HOME="/var/lib/greeter"
        usermod -d "$GREETER_HOME" greeter
    fi
    mkdir -p "$GREETER_HOME"
    chown -R greeter:greeter "$GREETER_HOME"
fi

# Ensure greetd user exists
if ! id -u greetd &>/dev/null; then
    echo "Creating greetd system user..."
    useradd --system --no-create-home --shell /usr/bin/nologin greetd
fi

# Ensure /var/lib/greetd exists and is owned by greetd
mkdir -p /var/lib/greetd
chown greetd:greetd /var/lib/greetd
# Add greeter to greetd group so it can write last_user.json
gpasswd -a greeter greetd
chmod 770 /var/lib/greetd

# Install greetd config
echo "Installing greetd config..."
mkdir -p /etc/greetd

# Clean up any pacnew file
if [ -f /etc/greetd/config.toml.pacnew ]; then
    rm /etc/greetd/config.toml.pacnew
fi

cat > /etc/greetd/config.toml << 'EOF'
[terminal]
vt = 1

[default_session]
command = "/usr/local/bin/hypr-greeter --bootstrap"
user = "greeter"
EOF

# Create systemd override
mkdir -p /etc/systemd/system/greetd.service.d

cat > /etc/systemd/system/greetd.service.d/override.conf << 'EOF'
[Service]
Environment="XDG_SESSION_TYPE=wayland"
TimeoutStartSec=30s
Restart=always
RestartSec=1s
KillMode=process
EOF

# Set permissions
chmod 644 /etc/greetd/config.toml
chmod 755 /usr/local/bin/hypr-greeter

# Enable greetd service
echo "Enabling greetd service..."
disable_conflicting_display_managers
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
