# Hypr-Greeter

A customizable TUI greeter for Hyprland and other Wayland compositors, built on [greetd](https://github.com/kennylevinsen/greetd).

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

## Keyboard Layout Configuration

**Keyboard layout is set via the greetd config, not via the greeter config.**

To set the keyboard layout for the greeter (Cage session), add the environment variable in your `/etc/greetd/config.toml`:

```toml
[default_session]
command = "env XKB_DEFAULT_LAYOUT=de cage -s -- hypr-greeter"
user = "greeter"
```
```bash
env XKB_DEFAULT_LAYOUT=de
```

Replace `de` with your desired layout (e.g., `us`, `ch`, etc.).

- This ensures the layout is set before Cage (the compositor for the greeter) starts.
- The `config.json` keyboard section is no longer used and can be removed.
- To set the layout for your desktop session (e.g., Hyprland), configure it in your session's own config.

---

## Installation

```bash
git clone https://github.com/obamosaurus/hypr-greeter
cd hypr-greeter
chmod +x install.sh
sudo bash install.sh
sudo systemctl start greetd
```

---

## Configuration

- UI, security, and session options are still configured in `config.json`.
- See the example `config.json` for all available options:

```json
{
  "default_user": "",
  "disable_autofill": false,
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
    },
    "field_width": 50,
    "field_height": 100,
    "field_spacing": 0,
    "top_to_clock_spacing": 15,
    "clock_to_fields_spacing": 0,
    "title": "hypr-greeter"
  },
  "security": {
    "clear_password_on_error": true,
    "mask_password": true,
    "input_timeout": 0
  }
}
```

---

## Screenshots

![First Test](image.png)

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

## Development

- **Build:** `cargo build` or `cargo build --release`
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

