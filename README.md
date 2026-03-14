# Hypr-Greeter

A customizable TUI greeter for Hyprland and other Wayland compositors, built on [greetd](https://github.com/kennylevinsen/greetd).

---

## Features

- **Multi-monitor support** — login mask on one monitor, solid background on others (via Hyprland)
- TUI interface (ratatui + foot terminal)
- Session selection (Hyprland, Sway, TTY, custom)
- Remembers last username
- Clock and date display
- Configurable UI and security options
- Secure password handling (no echo, optional masking)
- Simple TOML configuration

---

## Architecture

```
greetd → hypr-greeter-wrapper (bash) → start-hyprland → Hyprland (generated config)
                                                              ├─ foot --app-id=hypr-greeter -e hypr-greeter  (on login monitor)
                                                              └─ solid background color on all monitors
```

The wrapper script reads your TOML config, generates a minimal Hyprland config at runtime, and launches Hyprland as the temporary greeter compositor through `start-hyprland`. The TUI greeter runs inside a foot terminal on the designated login monitor.

`[[sessions]]` only controls the real session started after successful authentication. It does not change how the login mask itself is launched.

---

## Dependencies

- **greetd** — display manager daemon
- **hyprland** — compositor for multi-monitor support
- **foot** — terminal emulator for the TUI
- **rust/cargo** — for building from source

---

## Installation

```bash
git clone https://github.com/obamosaurus/hypr-greeter
cd hypr-greeter
chmod +x install.sh
sudo bash install.sh
sudo systemctl start greetd
```

If another display manager is already enabled, disable it first. On Arch with `ly`, run:

```bash
sudo systemctl disable --now ly.service
sudo systemctl enable greetd.service
sudo systemctl start greetd.service
```

---

## Configuration

Configuration is stored in `/etc/hypr-greeter/config.toml`.

### Full example

```toml
default_user = ""
disable_autofill = false

# Monitor setup (optional — omit entire section for auto-detect)
# Run `hyprctl monitors` from your session to find monitor names
[[monitors]]
name = "DP-1"
resolution = "2560x1440@144"
position = "0x0"
login = true        # show login mask on this monitor

[[monitors]]
name = "HDMI-A-1"
resolution = "1920x1080@60"
position = "2560x0"

[[sessions]]
name = "Hyprland"
command = "start-hyprland"

[[sessions]]
name = "Sway"
command = "sway"

[[sessions]]
name = "TTY"
command = "/bin/bash"

[ui]
show_clock = true
clock_format = "%H:%M"
show_date = true
date_format = "%A, %d %B %Y"
title = "hypr-greeter"
field_width = 50        # percentage of terminal width
field_spacing = 0       # rows between fields
top_spacing = 15        # rows from top to clock
clock_spacing = 0       # rows from clock to fields

[ui.colors]
background = "#1a1b26"
foreground = "#c0caf5"
focused = "#f7768e"
error = "#f7768e"

[security]
clear_password_on_error = true
mask_password = true
```

### Monitor examples

**Auto-detect (no config needed):**
Omit the `[[monitors]]` section entirely. Hyprland will auto-detect all connected monitors.

**Single monitor:**
```toml
[[monitors]]
name = "DP-1"
resolution = "1920x1080@60"
position = "0x0"
login = true
```

**Dual monitor:**
```toml
[[monitors]]
name = "DP-1"
resolution = "2560x1440@144"
position = "0x0"
login = true

[[monitors]]
name = "HDMI-A-1"
resolution = "1920x1080@60"
position = "2560x0"
```

**Triple monitor:**
```toml
[[monitors]]
name = "DP-1"
resolution = "1920x1080@60"
position = "0x0"

[[monitors]]
name = "DP-2"
resolution = "2560x1440@144"
position = "1920x0"
login = true

[[monitors]]
name = "HDMI-A-1"
resolution = "1920x1080@60"
position = "4480x0"
```

Run `hyprctl monitors` from your Hyprland session to find your monitor names.

---

## Keyboard Layout Configuration

Set keyboard layouts in `/etc/hypr-greeter/config.toml`:

```toml
[input]
kb_layout = "us,ch"
kb_options = "grp:alt_shift_toggle"
```

`ch` is the default Swiss German layout. With the example above, the login screen starts with `us` available and lets you switch to Swiss German using `Alt+Shift`.

If you need a specific Swiss variant, add `kb_variant`. For example:

```toml
[input]
kb_layout = "us,ch"
kb_variant = ",de_nodeadkeys"
kb_options = "grp:alt_shift_toggle"
```

The wrapper still accepts `XKB_DEFAULT_LAYOUT`, `XKB_DEFAULT_VARIANT`, and `XKB_DEFAULT_OPTIONS` from `/etc/greetd/config.toml`, but the greeter config is now the preferred place to manage this.

---

## Screenshots

![First Test](image.png)

---

## Uninstallation

```bash
chmod +x uninstall.sh
sudo ./uninstall.sh
```

---

## Development

- **Build:** `cargo build` or `cargo build --release`
- **Test wrapper:** `bash -x hypr-greeter-wrapper.sh` (inspect generated Hyprland config)

---

## License

MIT License

---

## Acknowledgments

- [greetd](https://github.com/kennylevinsen/greetd)
- [Hyprland](https://github.com/hyprwm/Hyprland)
- [foot](https://codeberg.org/dnkl/foot)
- [ratatui](https://github.com/ratatui-org/ratatui)

---

## Support

Open an issue or pull request on GitHub.
