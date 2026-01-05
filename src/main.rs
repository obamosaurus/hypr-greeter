// ~/hypr-greeter/src/main.rs
// Main entry point for hypr-greeter

mod config;
mod greetd_client;
mod ui;

use config::load_config;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::error::Error;
use std::io;
use ui::{App, Focus};

fn main() -> Result<(), Box<dyn Error>> {
    // Use a tokio runtime manually to avoid proc-macro issues
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = load_config()?;

    // Set keyboard layout if configured
    if let Some(ref layout) = config.keyboard_layout {
        std::env::set_var("XKB_DEFAULT_LAYOUT", layout);
    }

    // Setup terminal
    setup_terminal()?;
    
    // Create terminal backend
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new(config.clone());
    
    // Main event loop
    let result = run_app(&mut terminal, &mut app).await;
    
    // Cleanup terminal
    cleanup_terminal()?;
    
    // Handle any errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}

/// Setup terminal for TUI
fn setup_terminal() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

/// Cleanup terminal on exit
fn cleanup_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;
        
        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                // Navigation
                KeyCode::Tab => {
                    app.clear_error();
                    app.next_focus();
                }
                KeyCode::BackTab => {
                    app.clear_error();
                    app.prev_focus();
                }
                
                // Login
                KeyCode::Enter => {
                    if !app.username.is_empty() && !app.password.is_empty() {
                        // Attempt login
                        let login_result = greetd_client::login(
                            &app.username,
                            &app.password,
                            app.current_session_command(),
                        ).await;
                        match login_result {
                            Ok(_) => {
                                // Save last_user for fallback autofill
                                if let Err(e) = crate::config::save_last_user(&app.username) {
                                    eprintln!("Failed to save last_user: {}", e);
                                }
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                // Exit - greetd will handle the session
                                break;
                            }
                            Err(e) => {
                                // Try to cancel the session so user can retry
                                if let Ok(mut client) = greetd_client::GreetdClient::connect().await {
                                    let _ = client.cancel_session().await;
                                }
                                app.set_error(format!("Login failed: {}", e));
                            }
                        }
                    } else {
                        app.set_error("Please enter username and password".to_string());
                    }
                }
                
                // Text input
                KeyCode::Char(c) => {
                    app.clear_error();
                    match app.focus {
                        Focus::Username => app.username.push(c),
                        Focus::Password => app.password.push(c),
                        Focus::Session => {}
                    }
                }
                
                // Backspace
                KeyCode::Backspace => {
                    app.clear_error();
                    match app.focus {
                        Focus::Username => { app.username.pop(); }
                        Focus::Password => { app.password.pop(); }
                        Focus::Session => {}
                    }
                }
                
                // Session selection
                KeyCode::Left => {
                    if app.focus == Focus::Session {
                        app.clear_error();
                        app.prev_session();
                    }
                }
                KeyCode::Right => {
                    if app.focus == Focus::Session {
                        app.clear_error();
                        app.next_session();
                    }
                }
                
                // Exit (for debugging)
                KeyCode::Esc => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                }
                
                _ => {}
            }
        }
    }
    
    Ok(())
}