# Hypr-Greeter

A customizable TUI greeter for Hyprland and other Wayland compositors, built on [greetd](https://github.com/kennylevinsen/greetd).  
Supports multi-monitor setups (via [cage](https://github.com/Hjdskes/cage)), session selection, and secure password handling.

---

## Features

- Multi-monitor support (via cage)
- TUI interface (ratatui)
- Session selection (Hyprland, Sway, TTY, custom)
- Remembers last username
- Clock and date display
- Configurable UI and security options
- Secure password handling (no echo, optional masking)
- Easily extensible via config files

---

## Screenshots


![First Test](image.png)

---

## Installation

### Quick Install

```bash
sudo pacman -S greetd cage alacritty rust
git clone https://github.com/yourusername/hypr-greeter
cd hypr-greeter
chmod +x install.sh
sudo ./install.sh
sudo systemctl start greetd
```

### Manual Steps

1. **Build:**  
   `cargo build --release`
2. **Install binary:**  
   `sudo install -Dm755 target/release/hypr-greeter /usr/local/bin/hypr-greeter`
3. **Config directories:**  
   `sudo mkdir -p /etc/hypr-greeter /etc/greetd`
4. **Copy configs:**  
   `sudo cp config.json /etc/hypr-greeter/`  
   `sudo cp greetd/config.toml /etc/greetd/`
5. **Create greeter user:**  
   `sudo useradd -M -G video greeter`  
   `sudo passwd -d greeter`
6. **Enable greetd:**  
   `sudo systemctl enable --now greetd`

---

## Configuration

Hypr-Greeter is highly configurable.  
**All configuration is done via plain text files.**  
Below are the main files and what you can change in each.

### 1. Main Greeter Config: `/etc/hypr-greeter/config.json`

Controls UI, sessions, and security options.

Example:
```json
{
  "last_user": "", // Auto-filled after login
  "sessions": [
    { "name": "Hyprland", "command": "Hyprland" },
    { "name": "Sway", "command": "sway" },
    { "name": "TTY", "command": "/bin/bash" }
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
```

#### **What you can configure:**
- **Sessions:**  
  Add, remove, or edit sessions in the `"sessions"` array.  
  Each session has a `"name"` (displayed in the UI) and a `"command"` (run after login).  
  The `"command"` can be any executable, shell script, or command line.

- **UI:**  
  - Show/hide clock and date
  - Set clock/date format (uses [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) syntax)
  - Colors (for future use)

- **Security:**  
  - Mask password input
  - Clear password on error
  - Input timeout (not yet implemented)

---

### 3. greetd Config: `/etc/greetd/config.toml`

Controls how greetd launches the greeter.

Example:
```toml
[terminal]
vt = 1

[default_session]
command = "cage -s -- alacritty -e /usr/local/bin/hypr-greeter"
user = "greeter"
```

- You can change the terminal emulator (e.g., `foot`, `kitty`) or add arguments.
- The `user` should be `greeter` (or another dedicated user).

---

### 4. Systemd Override: `/etc/systemd/system/greetd.service.d/override.conf`

Advanced: tweak greetd's systemd service (env vars, dependencies, etc).

---

## Adding Sessions

You can add sessions in two ways:

1. **Edit `/etc/hypr-greeter/config.json`**  
   Add to the `"sessions"` array:
   ```json
   {
     "name": "My Custom WM",
     "command": "/usr/bin/my-wm-launcher"
   }
   ```
   - The `"command"` can be any executable, shell script, or command line.

2. **Edit `/etc/greetd/environments`**  
   (If supported by your setup/code)  
   Add a line for each session:
   ```
   Hyprland
   sway
   /usr/bin/my-wm-launcher
   /path/to/another-session.sh
   ```

---

## Usage

- **Tab**: Next field
- **Shift+Tab**: Previous field
- **Enter**: Login
- **←/→**: Change session (when focused)
- **Ctrl+Esc**: Exit (debug only)

---

## Troubleshooting

- **Greeter doesn't start:**  
  `journalctl -u greetd -f`
- **Multi-monitor issues:**  
  Ensure `cage` is installed and `greeter` is in the `video` group:  
  `sudo usermod -a -G video greeter`
- **Authentication fails:**  
  - Check greetd is running: `systemctl status greetd`
  - Socket exists: `ls -la /run/greetd.sock`
  - Test PAM: `pamtester greetd username authenticate`
- **Wrong resolution/scaling:**  
  Run manually:  
  `cage -d -- alacritty -e /usr/local/bin/hypr-greeter`

---

## Uninstallation

```bash
chmod +x uninstall.sh
sudo ./uninstall.sh
```
- Stops and disables greetd
- Removes the binary
- Optionally removes config files and greeter user

---

## Project Structure

```
hypr-greeter/
├── Cargo.toml
├── install.sh
├── uninstall.sh
├── README.md
└── src/
    ├── main.rs
    ├── config.rs
    ├── greetd_client.rs
    └── ui.rs
```

---

## Development

- **Build:** `cargo build` or `cargo build --release`
- **Run:** `cargo run`
- **Edit configs:** See above

---

## License

MIT License

---

## Acknowledgments

- [greetd](https://github.com/kennylevinsen/greetd)
- [cage](https://github.com/Hjdskes/cage)
- [ratatui](https://github.com/ratatui-org/ratatui)

---

## Support

Open an issue or pull request on GitHub.