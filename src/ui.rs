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
        let autofill = !config.disable_autofill.unwrap_or(false);
        let (username, focus) = if autofill {
            let user = config.default_user.as_ref().map(|u| u.as_str()).unwrap_or("");
            let username = if !user.is_empty() {
                user.to_string()
            } else if let Some(last) = config.last_user.as_ref() {
                if !last.is_empty() { last.clone() } else { String::new() }
            } else {
                String::new()
            };
            let focus = if username.is_empty() { Focus::Username } else { Focus::Password };
            (username, focus)
        } else {
            (String::new(), Focus::Username)
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
    
    /// Set error message and optionally clear password
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        if self.config.security.clear_password_on_error {
            self.password.clear();
        }
    }
}

/// Helper to parse hex colors from config
fn parse_hex_color(hex: &str) -> Color {
    Color::from_str(hex).unwrap_or(Color::White)
}

/// Main UI drawing function
pub fn draw(f: &mut Frame<'_>, app: &App) {
    let size = f.size();

    // Set background color from config color scheme
    let bg = parse_hex_color(&app.config.ui.colors.background);
    f.render_widget(
        Block::default().style(Style::default().bg(bg)),
        size,
    );

    // Sensible defaults and limits
    let min_height = 3u16;
    let max_height = 5u16;

    // Calculate field height as a percentage of terminal size, but clamp to min/max
    let height = app.config.ui.field_height.map(|v| ((min_height as u32 * v / 100) as u16).clamp(min_height, max_height)).unwrap_or(min_height);
    let spacing = app.config.ui.field_spacing.unwrap_or(1) as u16;
    let top_to_clock_spacing = app.config.ui.top_to_clock_spacing.unwrap_or(1) as u16;
    let clock_to_fields_spacing = app.config.ui.clock_to_fields_spacing.unwrap_or(1) as u16;
    let clock_date_height = if app.config.ui.show_clock && app.config.ui.show_date { 4 } else { 3 };

    // For width: 100 means half the terminal width (main monitor)
    let width = app.config.ui.field_width.map(|v| ((size.width as u32 * v / 200) as u16).clamp(20, size.width)).unwrap_or(size.width / 2);

    // Create main layout
    let chunks = if app.config.ui.show_clock || app.config.ui.show_date {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(top_to_clock_spacing), // Top to clock
                Constraint::Length(clock_date_height), // Clock/Date
                Constraint::Length(clock_to_fields_spacing), // Clock to fields
                Constraint::Length(height), // Username
                Constraint::Length(spacing), // Spacing
                Constraint::Length(height), // Password
                Constraint::Length(spacing), // Spacing
                Constraint::Length(height), // Session
                Constraint::Min(0),    // Error/Space
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(top_to_clock_spacing), // Top to fields
                Constraint::Length(height), // Username
                Constraint::Length(spacing), // Spacing
                Constraint::Length(height), // Password
                Constraint::Length(spacing), // Spacing
                Constraint::Length(height), // Session
                Constraint::Min(0),    // Error/Space
            ])
            .split(size)
    };

    let mut chunk_idx = 0;
    draw_title(f, chunks[chunk_idx], &app.config);
    chunk_idx += 1;
    chunk_idx += 1;
    if app.config.ui.show_clock || app.config.ui.show_date {
        draw_clock_date(f, chunks[chunk_idx], &app.config);
        chunk_idx += 1;
        chunk_idx += 1;
    }
    draw_username(f, chunks[chunk_idx], app, width);
    chunk_idx += 1;
    chunk_idx += 1;
    draw_password(f, chunks[chunk_idx], app, width);
    chunk_idx += 1;
    chunk_idx += 1;
    draw_session(f, chunks[chunk_idx], app, width);
    chunk_idx += 1;
    if let Some(ref error) = app.error_message {
        draw_error(f, chunks[chunk_idx], error);
    }
    // Always draw help at the bottom of the terminal
    draw_help(f, Rect {
        x: 0,
        y: size.height.saturating_sub(1),
        width: size.width,
        height: 1,
    });
}

/// Draw title
fn draw_title(f: &mut Frame<'_>, area: Rect, config: &Config) {
    let title_text = config.ui.title.as_deref().unwrap_or("Hypr-Greeter");
    let title = Paragraph::new(title_text)
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
fn draw_username(f: &mut Frame<'_>, area: Rect, app: &App, width: u16) {
    let style = get_field_style(app.focus == Focus::Username, &app.config.ui.colors);
    let username = Paragraph::new(app.username.as_str())
        .style(style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(style)
            .title("Username"))
        .wrap(Wrap { trim: true });
    let centered = centered_rect(width, area.height, area);
    f.render_widget(username, centered);
}

/// Draw password field
fn draw_password(f: &mut Frame<'_>, area: Rect, app: &App, width: u16) {
    let style = get_field_style(app.focus == Focus::Password, &app.config.ui.colors);
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
            .title("Password"))
        .wrap(Wrap { trim: true });
    let centered = centered_rect(width, area.height, area);
    f.render_widget(password, centered);
}

/// Draw session selector
fn draw_session(f: &mut Frame<'_>, area: Rect, app: &App, width: u16) {
    let style = get_field_style(app.focus == Focus::Session, &app.config.ui.colors);
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
            .title("Session"))
        .wrap(Wrap { trim: true });
    let centered = centered_rect(width, area.height, area);
    f.render_widget(session, centered);
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
    // Render directly in the provided area (which is already set to the bottom row)
    f.render_widget(help, area);
}

/// Get style for input fields based on focus and color scheme
fn get_field_style(focused: bool, colors: &crate::config::ColorScheme) -> Style {
    if focused {
        Style::default().fg(parse_hex_color(&colors.focused)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(parse_hex_color(&colors.foreground))
    }
}

/// Helper to center a rect of given width/height in parent area
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y;
    Rect { x, y, width: width.min(area.width), height: height.min(area.height) }
}