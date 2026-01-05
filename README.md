# Hypr-Greeter

A simple, clean TUI greeter for Hyprland and other Wayland compositors, built on [greetd](https://github.com/kennylevinsen/greetd).

---

## Features

- Clean and simple interface
- Multi-monitor support (login on primary, background on others)
- Session selection (Hyprland, Sway, TTY, etc.)
- Remembers last username
- Clock and date display
- Easy configuration

---

## Installation

```bash
git clone https://github.com/obamosaurus/hypr-greeter
cd hypr-greeter
chmod +x install.sh
sudo ./install.sh
sudo systemctl start greetd
```

The install script works on most Linux distributions (Arch, Debian/Ubuntu, Fedora, openSUSE).

---

## Configuration

Edit `/etc/hypr-greeter/config.json`:

```json
{
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
  "background": "#1a1b26",
  "show_clock": true,
  "title": "Welcome"
}
```

### Configuration Options

- **sessions**: List of available sessions/window managers
  - **name**: Display name
  - **command**: Command to execute
- **background**: Background color (hex format)
- **show_clock**: Show clock and date (true/false)
- **title**: Title text displayed at the top

---

## Keyboard Layout

Set keyboard layout via greetd config (`/etc/greetd/config.toml`):

```toml
[default_session]
command = "env XKB_DEFAULT_LAYOUT=de cage -s -- /usr/local/bin/hypr-greeter"
user = "greeter"
```

Replace `de` with your layout (e.g., `us`, `ch`, etc.).

---

## Multi-Monitor Support

Multi-monitor support is handled automatically by cage (the Wayland compositor):
- The login interface appears on the primary monitor
- Other monitors display the background color
- Works with any resolution and monitor configuration

---

## Uninstallation

```bash
chmod +x uninstall.sh
sudo ./uninstall.sh
```

---

## License

MIT License

---

## Acknowledgments

- [greetd](https://github.com/kennylevinsen/greetd)
- [cage](https://github.com/Hjdskes/cage)
- [ratatui](https://github.com/ratatui-org/ratatui)

