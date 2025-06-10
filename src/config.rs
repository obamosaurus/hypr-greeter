// ~/hypr-greeter/src/config.rs
// Configuration handling module

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Last logged in username - will be pre-filled
    pub last_user: String,
    
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
            last_user: String::new(),
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
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        // Remove comments for JSON parsing
        let cleaned = remove_json_comments(&content);
        Ok(serde_json::from_str(&cleaned)?)
    } else {
        Ok(Config::default())
    }
}

/// Save configuration to disk
pub fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
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