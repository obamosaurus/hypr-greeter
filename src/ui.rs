// ~/hypr-greeter/src/ui.rs
// Terminal UI rendering module

use crate::config::Config;
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Application state for the UI
pub struct App {
    pub username: String,
    pub password: String,
    pub selected_session: usize,
    pub focus: Focus,
    pub error_message: Option<String>,
    pub config: Config,
}

/// Which field is currently focused
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Username,
    Password,
    Session,
}

impl App {
    /// Create new app state
    pub fn new(config: Config) -> Self {
        let username = config.last_user.clone();
        
        Self {
            username,
            password: String::new(),
            selected_session: 0,
            focus: if config.last_user.is_empty() { 
                Focus::Username 
            } else { 
                Focus::Password 
            },
            error_message: None,
            config,
        }
    }
    
    /// Move focus to next field
    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Username => Focus::Password,
            Focus::Password => Focus::Session,
            Focus::Session => Focus::Username,
        };
    }
    
    /// Move focus to previous field
    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Username => Focus::Session,
            Focus::Password => Focus::Username,
            Focus::Session => Focus::Password,
        };
    }
    
    /// Select next session
    pub fn next_session(&mut self) {
        if self.selected_session < self.config.sessions.len() - 1 {
            self.selected_session += 1;
        }
    }
    
    /// Select previous session
    pub fn prev_session(&mut self) {
        if self.selected_session > 0 {
            self.selected_session -= 1;
        }
    }
    
    /// Get current session command
    pub fn current_session_command(&self) -> &str {
        &self.config.sessions[self.selected_session].command
    }
    
    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
    
    /// Set error message and optionally clear password
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        if self.config.security.clear_password_on_error {
            self.password.clear();
        }
    }
}

/// Main UI drawing function
pub fn draw(f: &mut Frame<'_>, app: &App) {
    let size = f.size();
    
    // Create main layout
    let chunks = if app.config.ui.show_clock || app.config.ui.show_date {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(if app.config.ui.show_clock && app.config.ui.show_date { 4 } else { 3 }), // Clock/Date
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Session
                Constraint::Min(0),    // Error/Space
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Session
                Constraint::Min(0),    // Error/Space
            ])
            .split(size)
    };
    
    let mut chunk_idx = 0;
    
    // Title
    draw_title(f, chunks[chunk_idx]);
    chunk_idx += 1;
    
    // Clock and Date
    if app.config.ui.show_clock || app.config.ui.show_date {
        draw_clock_date(f, chunks[chunk_idx], &app.config);
        chunk_idx += 1;
    }
    
    // Username field
    draw_username(f, chunks[chunk_idx], app);
    chunk_idx += 1;
    
    // Password field
    draw_password(f, chunks[chunk_idx], app);
    chunk_idx += 1;
    
    // Session selector
    draw_session(f, chunks[chunk_idx], app);
    chunk_idx += 1;
    
    // Error message
    if let Some(ref error) = app.error_message {
        draw_error(f, chunks[chunk_idx], error);
    }
    
    // Help text at bottom
    draw_help(f, size);
}

/// Draw title
fn draw_title(f: &mut Frame<'_>, area: Rect) {
    let title = Paragraph::new("Hyprland Greeter")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, area);
}

/// Draw clock and date
fn draw_clock_date(f: &mut Frame<'_>, area: Rect, config: &Config) {
    let now = Local::now();
    let mut text = Vec::new();
    
    if config.ui.show_clock {
        let clock = now.format(&config.ui.clock_format).to_string();
        text.push(Line::from(vec![
            Span::styled(clock, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        ]));
    }
    
    if config.ui.show_date {
        let date = now.format(&config.ui.date_format).to_string();
        text.push(Line::from(vec![
            Span::styled(date, Style::default().fg(Color::Gray))
        ]));
    }
    
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}

/// Draw username field
fn draw_username(f: &mut Frame<'_>, area: Rect, app: &App) {
    let style = get_field_style(app.focus == Focus::Username);
    let username = Paragraph::new(app.username.as_str())
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Username"));
    f.render_widget(username, area);
}

/// Draw password field
fn draw_password(f: &mut Frame<'_>, area: Rect, app: &App) {
    let style = get_field_style(app.focus == Focus::Password);
    let password_display = if app.config.security.mask_password {
        "*".repeat(app.password.len())
    } else {
        app.password.clone()
    };
    
    let password = Paragraph::new(password_display)
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Password"));
    f.render_widget(password, area);
}

/// Draw session selector
fn draw_session(f: &mut Frame<'_>, area: Rect, app: &App) {
    let style = get_field_style(app.focus == Focus::Session);
    let session_text = if app.focus == Focus::Session {
        format!("< {} >", app.config.sessions[app.selected_session].name)
    } else {
        app.config.sessions[app.selected_session].name.clone()
    };
    
    let session = Paragraph::new(session_text)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Session"));
    f.render_widget(session, area);
}

/// Draw error message
fn draw_error(f: &mut Frame<'_>, area: Rect, error: &str) {
    let error_widget = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(error_widget, area);
}

/// Draw help text
fn draw_help(f: &mut Frame<'_>, area: Rect) {
    let help_text = "Tab: Next Field | Shift+Tab: Previous Field | ←/→: Change Session | Enter: Login | Esc: Exit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    
    let help_area = Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    
    f.render_widget(help, help_area);
}

/// Get style for input fields based on focus
fn get_field_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}