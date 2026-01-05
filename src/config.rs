// ~/hypr-greeter/src/config.rs
// Configuration handling module

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Available sessions/WMs
    pub sessions: Vec<Session>,
    
    /// Background color (hex format)
    #[serde(default = "default_background")]
    pub background: String,
    
    /// Show clock and date
    #[serde(default = "default_true")]
    pub show_clock: bool,
    
    /// Title text for the greeter
    #[serde(default = "default_title")]
    pub title: String,

    /// Keyboard layout (e.g., "us", "de", "fr", etc.)
    #[serde(default)]
    pub keyboard_layout: Option<String>,

    /// Last logged in username - loaded from separate file
    #[serde(skip)]
    pub last_user: Option<String>,
}

/// Session/Window Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Display name in the UI
    pub name: String,
    
    /// Command to execute
    pub command: String,
}

fn default_background() -> String {
    "#1a1b26".to_string()
}

fn default_true() -> bool {
    true
}

fn default_title() -> String {
    "Welcome".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
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
            background: default_background(),
            show_clock: true,
            title: default_title(),
            keyboard_layout: None,
            last_user: None,
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
        serde_json::from_str(&content)?
    } else {
        Config::default()
    };
    
    // Load last_user from file for autofill
    let user_file = std::path::PathBuf::from("/var/lib/greetd/last_user.json");
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