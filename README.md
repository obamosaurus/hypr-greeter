# Hypr-Greeter

A custom TUI greeter for Hyprland with multi-monitor support, built on greetd.

## Features

- ✅ **Multi-monitor support** with different resolutions (via Cage)
- ✅ **GUI/TUI hybrid** interface
- ✅ **Direct session launch** on current TTY
- ✅ **Wayland-only** (no X11 dependencies)
- ✅ **Remembers last username**
- ✅ **Proper scaling** on each monitor
- ✅ **Session selection** (Hyprland/Sway/TTY)
- ✅ **Clock and date display**
- ✅ **Configurable UI** via JSON config
- ✅ **Secure password handling**

## Screenshots

```
┌─────────────────────────────────────┐
│        Hyprland Greeter             │
│                                     │
│            14:32                    │
│      Monday, 10 June 2025           │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Username                        │ │
│ │ user                            │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Password                        │ │
│ │ ********                        │ │
│ └─────────────────────────────────┘ │
│                                     │
│ ┌─────────────────────────────────┐ │
│ │ Session                         │ │
│ │         < Hyprland >            │ │
│ └─────────────────────────────────┘ │
│                                     │
└─────────────────────────────────────┘
```

## Requirements

- Arch Linux (or compatible distribution)
- greetd
- cage (for multi-monitor support)
- alacritty (or another terminal emulator)
- Rust toolchain (for building from source)

## Installation

### Quick Install

```bash
# Install dependencies
sudo pacman -S greetd cage alacritty rust

# Clone the repository
git clone https://github.com/yourusername/hypr-greeter
cd hypr-greeter

# Run the installation script
chmod +x install.sh
sudo ./install.sh

# Start the greeter
sudo systemctl start greetd
```

### Manual Installation

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Install the binary:**
   ```bash
   sudo install -Dm755 target/release/hypr-greeter /usr/local/bin/hypr-greeter
   ```

3. **Create config directories:**
   ```bash
   sudo mkdir -p /etc/hypr-greeter
   sudo mkdir -p /etc/greetd
   ```

4. **Install config files:**
   ```bash
   # Copy the default config
   sudo cp config.json /etc/hypr-greeter/
   
   # Copy greetd config
   sudo cp greetd/config.toml /etc/greetd/
   ```

5. **Create greeter user:**
   ```bash
   sudo useradd -M -G video greeter
   sudo passwd -d greeter
   ```

6. **Enable and start greetd:**
   ```bash
   sudo systemctl enable --now greetd
   ```

## Configuration

### Main Configuration (`/etc/hypr-greeter/config.json`)

```json
{
  "last_user": "",              // Automatically saved
  "sessions": [                 // Available sessions
    {
      "name": "Hyprland",
      "command": "Hyprland"
    },
    {
      "name": "Sway",
      "command": "sway"
    }
  ],
  "ui": {
    "show_clock": true,         // Display clock
    "clock_format": "%H:%M",    // 24h format
    "show_date": true,          // Display date
    "date_format": "%A, %d %B %Y"
  },
  "security": {
    "clear_password_on_error": true,
    "mask_password": true,
    "input_timeout": 0          // 0 = disabled
  }
}
```

### greetd Configuration (`/etc/greetd/config.toml`)

```toml
[terminal]
vt = 1

[default_session]
# Multi-monitor support with cage
command = "cage -s -- alacritty -e /usr/local/bin/hypr-greeter"
user = "greeter"

# Alternative terminal emulators:
# command = "cage -s -- foot -e /usr/local/bin/hypr-greeter"
# command = "cage -s -- kitty -e /usr/local/bin/hypr-greeter"
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Enter` | Login |
| `←`/`→` | Change session (when focused) |
| `Ctrl+Esc` | Exit (debug only) |

### Adding New Sessions

Edit `/etc/hypr-greeter/config.json` and add to the sessions array:

```json
{
  "name": "Your WM",
  "command": "your-wm-command"
}
```

## Project Structure

```
hypr-greeter/
├── Cargo.toml              # Rust dependencies
├── install.sh              # Installation script
├── uninstall.sh            # Uninstallation script
├── README.md               # This file
└── src/
    ├── main.rs             # Entry point and event loop
    ├── config.rs           # Configuration handling
    ├── greetd_client.rs    # greetd IPC communication
    └── ui.rs               # Terminal UI rendering
```

## Troubleshooting

### Greeter doesn't start

Check the logs:
```bash
journalctl -u greetd -f
```

### Multi-monitor issues

Ensure cage is installed and the greeter user is in the `video` group:
```bash
sudo usermod -a -G video greeter
```

### Authentication fails

- Check that greetd is running: `systemctl status greetd`
- Verify the socket exists: `ls -la /run/greetd.sock`
- Test PAM configuration: `pamtester greetd username authenticate`

### Wrong resolution/scaling

This is handled by cage. You can debug by running cage manually:
```bash
cage -d -- alacritty -e /usr/local/bin/hypr-greeter
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run directly (for testing)
cargo run
```

### Code Organization

- **main.rs**: Application entry point, terminal setup, main event loop
- **config.rs**: Configuration structures and file handling
- **greetd_client.rs**: Async greetd IPC protocol implementation
- **ui.rs**: TUI rendering with ratatui, application state

### Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Uninstallation

```bash
chmod +x uninstall.sh
sudo ./uninstall.sh
```

This will:
- Stop and disable greetd
- Remove the binary
- Optionally remove config files
- Optionally remove the greeter user

## Future Plans

- [ ] Background image support
- [ ] Animations and transitions
- [ ] Theme customization
- [ ] Power management options (shutdown/reboot)
- [ ] Multiple monitor layouts (mirror/extend)
- [ ] User avatar support
- [ ] Fingerprint authentication
- [ ] Accessibility features

## License

MIT License - See LICENSE file for details

## Acknowledgments

- [greetd](https://github.com/kennylevinsen/greetd) - The minimal greeter daemon
- [cage](https://github.com/Hjdskes/cage) - Wayland kiosk compositor
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework

## Support

For issues, questions, or contributions, please use the GitHub issue tracker.