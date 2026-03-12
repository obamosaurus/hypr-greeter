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
    // Install panic hook to restore terminal on panic
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        default_hook(info);
    }));

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config.clone());

    let result = run_app(&mut terminal, &mut app).await;

    cleanup_terminal()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn setup_terminal() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn cleanup_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Tab => {
                    app.clear_error();
                    app.next_focus();
                }
                KeyCode::BackTab => {
                    app.clear_error();
                    app.prev_focus();
                }

                KeyCode::Enter => {
                    if !app.username.is_empty() && !app.password.is_empty() {
                        let session_cmd = match app.current_session_command() {
                            Some(cmd) => cmd.to_string(),
                            None => {
                                app.set_error("No sessions configured".to_string());
                                continue;
                            }
                        };

                        let login_result = greetd_client::login(
                            &app.username,
                            &app.password,
                            &session_cmd,
                        ).await;
                        match login_result {
                            Ok(_) => {
                                if let Err(e) = crate::config::save_last_user(&app.username) {
                                    eprintln!("Failed to save last_user: {}", e);
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                break;
                            }
                            Err(e) => {
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

                KeyCode::Char(c) => {
                    app.clear_error();
                    match app.focus {
                        Focus::Username => app.username.push(c),
                        Focus::Password => app.password.push(c),
                        Focus::Session => {}
                    }
                }

                KeyCode::Backspace => {
                    app.clear_error();
                    match app.focus {
                        Focus::Username => { app.username.pop(); }
                        Focus::Password => { app.password.pop(); }
                        Focus::Session => {}
                    }
                }

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
