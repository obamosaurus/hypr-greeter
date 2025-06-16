// ~/hypr-greeter/src/config.rs
// Configuration handling module

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Last logged in username - will be used for autofill fallback
    pub last_user: Option<String>,
    /// Username to autofill at startup (if set, overrides last_user)
    pub default_user: Option<String>,
    /// If true, disables autofilling username at startup
    pub disable_autofill: Option<bool>,

    /// Available sessions/WMs
    pub sessions: Vec<Session>,
    
    /// UI configuration
    pub ui: UiConfig,
    
    /// Security settings
    pub security: SecurityConfig,
}

/// Session/Window Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Display name in the UI
    pub name: String,
    
    /// Command to execute
    pub command: String,
}

/// UI customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show clock in the UI
    pub show_clock: bool,
    /// Clock format (chrono format string)
    pub clock_format: String,
    /// Show date under clock
    pub show_date: bool,
    /// Date format (chrono format string)
    pub date_format: String,
    /// Color scheme (for future use)
    pub colors: ColorScheme,

    // Login field dimensions (percentage scale: 100 = normal, 50 = half, 200 = double, etc)
    pub field_width: Option<u32>,    // percent scale
    pub field_height: Option<u32>,   // percent scale

    /// Spacing between input fields (in rows/lines)
    pub field_spacing: Option<u32>,

    /// Spacing from top of screen to clock/date (in rows/lines)
    pub top_to_clock_spacing: Option<u32>,
    /// Spacing from clock/date to input fields (in rows/lines)
    pub clock_to_fields_spacing: Option<u32>,

    /// Title text for the greeter
    pub title: Option<String>,
}

/// Color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: String,
    pub foreground: String,
    pub focused: String,
    pub error: String,
    pub success: String,
}

/// Security-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Clear password field after failed attempt
    pub clear_password_on_error: bool,
    
    /// Show asterisks for password
    pub mask_password: bool,
    
    /// Timeout in seconds before clearing input (0 = disabled)
    pub input_timeout: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_user: None,
            default_user: None,
            disable_autofill: None,
            sessions: vec![
                Session {
                    name: "Hyprland".to_string(),
                    command: "Hyprland".to_string(),
                },
                Session {
                    name: "Sway".to_string(),
                    command: "sway".to_string(),
                },
                Session {
                    name: "TTY".to_string(),
                    command: "/bin/bash".to_string(),
                },
            ],
            ui: UiConfig {
                show_clock: true,
                clock_format: "%H:%M".to_string(),
                show_date: true,
                date_format: "%A, %d %B %Y".to_string(),
                colors: ColorScheme {
                    background: "#1a1b26".to_string(),
                    foreground: "#c0caf5".to_string(),
                    focused: "#f7768e".to_string(),
                    error: "#f7768e".to_string(),
                    success: "#9ece6a".to_string(),
                },
                field_width: Some(100), // 100% (normal)
                field_height: Some(100), // 100% (normal)
                field_spacing: Some(1),
                top_to_clock_spacing: Some(100), // 100% (normal, interpreted as 1 row)
                clock_to_fields_spacing: Some(100), // 100% (normal, interpreted as 1 row)
                title: Some("Hyprland Greeter".to_string()),
            },
            security: SecurityConfig {
                clear_password_on_error: true,
                mask_password: true,
                input_timeout: 0,
            },
        }
    }
}

/// Get the configuration file path
pub fn config_path() -> PathBuf {
    // Try system config first
    let system_config = PathBuf::from("/etc/hypr-greeter/config.json");
    if system_config.exists() {
        return system_config;
    }
    
    // Fall back to user config
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("hypr-greeter")
        .join("config.json")
}

/// Load configuration from disk
pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let path = config_path();
    let mut config = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        // Remove comments for JSON parsing
        let cleaned = remove_json_comments(&content);
        serde_json::from_str(&cleaned)?
    } else {
        Config::default()
    };
    // Optionally load last_user from file for fallback autofill
    let user_file = std::path::PathBuf::from("/var/lib/greetd/last_user.json");
    if user_file.exists() {
        let content = std::fs::read_to_string(&user_file)?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(last_user) = json.get("last_user").and_then(|v| v.as_str()) {
                config.last_user = Some(last_user.to_string());
            }
        }
    }
    Ok(config)
}

/// Remove // style comments from JSON
fn remove_json_comments(json: &str) -> String {
    json.lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Save the last logged in username to a file
pub fn save_last_user(username: &str) -> Result<(), Box<dyn Error>> {
    let user_file = std::path::PathBuf::from("/var/lib/greetd/last_user.json");
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