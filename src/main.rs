// ~/hypr-greeter/src/main.rs
// Main entry point for hypr-greeter

mod config;
mod greetd_client;
mod ui;

use config::{load_config, save_config};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let mut config = load_config()?;
    
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
                        match greetd_client::login(
                            &app.username,
                            &app.password,
                            app.current_session_command(),
                        ).await {
                            Ok(_) => {
                                // Save last username
                                app.config.last_user = app.username.clone();
                                save_config(&app.config)?;
                                
                                // Exit - greetd will handle the session
                                break;
                            }
                            Err(e) => {
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