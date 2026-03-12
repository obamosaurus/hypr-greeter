use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Last logged in username (loaded from state file, not from config)
    #[serde(skip)]
    pub last_user: Option<String>,
    /// Username to autofill at startup (if set, overrides last_user)
    #[serde(default)]
    pub default_user: Option<String>,
    /// If true, disables autofilling username at startup
    #[serde(default)]
    pub disable_autofill: Option<bool>,

    /// Monitor configuration for multi-monitor support
    #[serde(default)]
    pub monitors: Vec<MonitorConfig>,

    /// Available sessions/WMs
    #[serde(default = "default_sessions")]
    pub sessions: Vec<Session>,

    /// Keyboard input configuration passed through to Hyprland
    #[serde(default)]
    pub input: InputConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,

    /// Security settings
    #[serde(default)]
    pub security: SecurityConfig,
}

/// Monitor configuration for Hyprland
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Monitor name (e.g. "DP-1", "HDMI-A-1")
    pub name: String,
    /// Resolution string (e.g. "2560x1440@144") — defaults to "preferred"
    #[serde(default = "default_resolution")]
    pub resolution: String,
    /// Position string (e.g. "0x0") — defaults to "auto"
    #[serde(default = "default_position")]
    pub position: String,
    /// Scale factor — defaults to 1.0
    #[serde(default = "default_scale")]
    pub scale: f64,
    /// Whether to show the login mask on this monitor
    #[serde(default)]
    pub login: bool,
}

fn default_resolution() -> String {
    "preferred".to_string()
}

fn default_position() -> String {
    "auto".to_string()
}

fn default_scale() -> f64 {
    1.0
}

/// Session/Window Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Display name in the UI
    pub name: String,
    /// Command to execute
    pub command: String,
}

/// Keyboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputConfig {
    /// Comma-separated XKB layouts, e.g. "us,ch"
    #[serde(default)]
    pub kb_layout: String,
    /// Comma-separated XKB variants, matching kb_layout positions
    #[serde(default)]
    pub kb_variant: String,
    /// XKB options, e.g. "grp:alt_shift_toggle"
    #[serde(default)]
    pub kb_options: String,
}

/// UI customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show clock in the UI
    #[serde(default = "default_true")]
    pub show_clock: bool,
    /// Clock format (chrono format string)
    #[serde(default = "default_clock_format")]
    pub clock_format: String,
    /// Show date under clock
    #[serde(default = "default_true")]
    pub show_date: bool,
    /// Date format (chrono format string)
    #[serde(default = "default_date_format")]
    pub date_format: String,
    /// Color scheme
    #[serde(default)]
    pub colors: ColorScheme,

    /// Field width as percentage of terminal width
    #[serde(default = "default_field_width")]
    pub field_width: u32,

    /// Spacing between input fields (in rows)
    #[serde(default)]
    pub field_spacing: u32,

    /// Spacing from top of screen to clock/date (in rows)
    #[serde(default = "default_top_spacing")]
    pub top_spacing: u32,
    /// Spacing from clock/date to input fields (in rows)
    #[serde(default)]
    pub clock_spacing: u32,

    /// Title text for the greeter
    #[serde(default = "default_title")]
    pub title: String,
}

/// Color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_fg")]
    pub foreground: String,
    #[serde(default = "default_accent")]
    pub focused: String,
    #[serde(default = "default_accent")]
    pub error: String,
}

/// Security-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Clear password field after failed attempt
    #[serde(default = "default_true")]
    pub clear_password_on_error: bool,
    /// Show asterisks for password
    #[serde(default = "default_true")]
    pub mask_password: bool,
}

// Default value helpers
fn default_true() -> bool { true }
fn default_clock_format() -> String { "%H:%M".to_string() }
fn default_date_format() -> String { "%A, %d %B %Y".to_string() }
fn default_field_width() -> u32 { 50 }
fn default_top_spacing() -> u32 { 15 }
fn default_title() -> String { "hypr-greeter".to_string() }
fn default_bg() -> String { "#1a1b26".to_string() }
fn default_fg() -> String { "#c0caf5".to_string() }
fn default_accent() -> String { "#f7768e".to_string() }

fn default_sessions() -> Vec<Session> {
    vec![
        Session { name: "Hyprland".to_string(), command: "start-hyprland".to_string() },
        Session { name: "Sway".to_string(), command: "sway".to_string() },
        Session { name: "TTY".to_string(), command: "/bin/bash".to_string() },
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_user: None,
            default_user: None,
            disable_autofill: None,
            monitors: Vec::new(),
            sessions: default_sessions(),
            input: InputConfig::default(),
            ui: UiConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_clock: true,
            clock_format: default_clock_format(),
            show_date: true,
            date_format: default_date_format(),
            colors: ColorScheme::default(),
            field_width: default_field_width(),
            field_spacing: 0,
            top_spacing: default_top_spacing(),
            clock_spacing: 0,
            title: default_title(),
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: default_bg(),
            foreground: default_fg(),
            focused: default_accent(),
            error: default_accent(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            clear_password_on_error: true,
            mask_password: true,
        }
    }
}

impl Config {
    /// Get the name of the monitor designated for login, if any
    #[allow(dead_code)]
    pub fn login_monitor_name(&self) -> Option<&str> {
        self.monitors
            .iter()
            .find(|m| m.login)
            .or(self.monitors.first())
            .map(|m| m.name.as_str())
    }

    pub fn layout_switch_hint(&self) -> Option<&'static str> {
        let layout_count = self
            .input
            .kb_layout
            .split(',')
            .filter(|layout| !layout.trim().is_empty())
            .count();

        if layout_count < 2 {
            return None;
        }

        let options = self.input.kb_options.as_str();
        if options.contains("grp:alt_shift_toggle") {
            Some("Alt+Shift: Switch Layout")
        } else if options.contains("grp:ctrl_shift_toggle") {
            Some("Ctrl+Shift: Switch Layout")
        } else if options.contains("grp:win_space_toggle") {
            Some("Super+Space: Switch Layout")
        } else if options.contains("grp:caps_toggle") {
            Some("Caps Lock: Switch Layout")
        } else {
            Some("Use configured XKB shortcut to switch layout")
        }
    }
}

/// Get the configuration file path
pub fn config_path() -> PathBuf {
    // Try system config first
    let system_config = PathBuf::from("/etc/hypr-greeter/config.toml");
    if system_config.exists() {
        return system_config;
    }

    // Fall back to user config
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("hypr-greeter")
        .join("config.toml")
}

/// Load configuration from disk
pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let path = config_path();
    let mut config: Config = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        toml::from_str(&content)?
    } else {
        Config::default()
    };

    // Load last_user from state file for fallback autofill
    let user_file = PathBuf::from("/var/lib/greetd/last_user.json");
    if user_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&user_file) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(last_user) = json.get("last_user").and_then(|v| v.as_str()) {
                    config.last_user = Some(last_user.to_string());
                }
            }
        }
    }

    Ok(config)
}

/// Save the last logged in username to a file
pub fn save_last_user(username: &str) -> Result<(), Box<dyn Error>> {
    let user_file = PathBuf::from("/var/lib/greetd/last_user.json");
    let content = serde_json::to_string_pretty(&serde_json::json!({"last_user": username}))?;
    if let Some(parent) = user_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = std::fs::File::create(user_file)?;
    file.write_all(content.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}
