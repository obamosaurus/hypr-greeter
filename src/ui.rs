// ~/hypr-greeter/src/ui.rs
// Terminal UI rendering module

use crate::config::Config;
use chrono::Local;
use std::str::FromStr;
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
        // Auto-fill last user if available
        let username = config.last_user.clone().unwrap_or_default();
        let focus = if username.is_empty() { 
            Focus::Username 
        } else { 
            Focus::Password 
        };
        
        Self {
            username,
            password: String::new(),
            selected_session: 0,
            focus,
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
    
    /// Set error message and clear password
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.password.clear();
    }
}

/// Helper to parse hex colors from config
fn parse_hex_color(hex: &str) -> Color {
    Color::from_str(hex).unwrap_or(Color::White)
}

/// Main UI drawing function
pub fn draw(f: &mut Frame<'_>, app: &App) {
    let size = f.size();

    // Set background color
    let bg = parse_hex_color(&app.config.background);
    f.render_widget(
        Block::default().style(Style::default().bg(bg)),
        size,
    );

    // Simple fixed layout - no complex calculations
    let chunks = if app.config.show_clock {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(2),  // Spacing
                Constraint::Length(3),  // Clock/Date
                Constraint::Length(3),  // Spacing
                Constraint::Length(3),  // Username
                Constraint::Length(1),  // Spacing
                Constraint::Length(3),  // Password
                Constraint::Length(1),  // Spacing
                Constraint::Length(3),  // Session
                Constraint::Min(2),     // Error/Space
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(5),  // Spacing
                Constraint::Length(3),  // Username
                Constraint::Length(1),  // Spacing
                Constraint::Length(3),  // Password
                Constraint::Length(1),  // Spacing
                Constraint::Length(3),  // Session
                Constraint::Min(2),     // Error/Space
            ])
            .split(size)
    };

    let mut idx = 0;
    draw_title(f, chunks[idx], &app.config);
    idx += 2; // Skip title and spacing
    
    if app.config.show_clock {
        draw_clock_date(f, chunks[idx]);
        idx += 2; // Skip clock and spacing
    }
    
    draw_username(f, chunks[idx], app);
    idx += 2; // Skip field and spacing
    
    draw_password(f, chunks[idx], app);
    idx += 2; // Skip field and spacing
    
    draw_session(f, chunks[idx], app);
    idx += 1;
    
    if let Some(ref error) = app.error_message {
        draw_error(f, chunks[idx], error);
    }
    
    // Help at the bottom
    draw_help(f, Rect {
        x: 0,
        y: size.height.saturating_sub(1),
        width: size.width,
        height: 1,
    });
}

/// Draw title
fn draw_title(f: &mut Frame<'_>, area: Rect, config: &Config) {
    let title = Paragraph::new(config.title.as_str())
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

/// Draw clock and date
fn draw_clock_date(f: &mut Frame<'_>, area: Rect) {
    let now = Local::now();
    let clock = now.format("%H:%M").to_string();
    let date = now.format("%A, %d %B %Y").to_string();
    
    let text = vec![
        Line::from(Span::styled(clock, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(date, Style::default().fg(Color::Gray))),
    ];
    
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Draw username field
fn draw_username(f: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Focus::Username;
    let style = if focused {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let widget = Paragraph::new(app.username.as_str())
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Username"));
    
    let centered = centered_rect(50, area);
    f.render_widget(widget, centered);
}

/// Draw password field
fn draw_password(f: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Focus::Password;
    let style = if focused {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let password_display = "*".repeat(app.password.len());
    let widget = Paragraph::new(password_display)
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Password"));
    
    let centered = centered_rect(50, area);
    f.render_widget(widget, centered);
}

/// Draw session selector
fn draw_session(f: &mut Frame<'_>, area: Rect, app: &App) {
    let focused = app.focus == Focus::Session;
    let style = if focused {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let session_text = if focused {
        format!("< {} >", app.config.sessions[app.selected_session].name)
    } else {
        app.config.sessions[app.selected_session].name.clone()
    };
    
    let widget = Paragraph::new(session_text)
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Session"));
    
    let centered = centered_rect(50, area);
    f.render_widget(widget, centered);
}

/// Draw error message
fn draw_error(f: &mut Frame<'_>, area: Rect, error: &str) {
    let widget = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(widget, area);
}

/// Draw help text
fn draw_help(f: &mut Frame<'_>, area: Rect) {
    let help_text = "Tab: Next | Shift+Tab: Previous | ←/→: Session | Enter: Login";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, area);
}

/// Center a widget horizontally with percentage width
fn centered_rect(percent_width: u16, area: Rect) -> Rect {
    let width = (area.width * percent_width) / 100;
    let width = width.max(30).min(area.width);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    Rect {
        x,
        y: area.y,
        width,
        height: area.height,
    }
}