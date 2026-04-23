use std::io;
use std::time::Duration;

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
    }
}

struct App {
    initial_message: String,
    messages: Vec<String>,
    input: String,
    scroll: usize,
    follow_latest: bool,
    should_exit: bool,
}

impl App {
    fn new() -> Self {
        let initial_message = "Type anything, press Enter to submit, clear to reset, or type exit to quit.".to_string();
        Self {
            messages: vec![initial_message.clone()],
            initial_message,
            input: String::new(),
            scroll: 0,
            follow_latest: true,
            should_exit: false,
        }
    }

    fn reset_messages(&mut self) {
        self.messages.clear();
        self.messages.push(self.initial_message.clone());
        self.scroll = 0;
        self.follow_latest = true;
    }

    fn append_message(&mut self, message: String) {
        self.messages.push(message);
        self.follow_latest = true;
    }

    fn scroll_up(&mut self) {
        self.follow_latest = false;
        self.scroll = self.scroll.saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        self.follow_latest = false;
        self.scroll = self.scroll.saturating_add(1).min(self.messages.len().saturating_sub(1));
    }

    fn sync_scroll(&mut self, viewport_height: u16) {
        let content_height = self.messages.len();
        let visible_rows = viewport_height.saturating_sub(2) as usize;

        if visible_rows == 0 || content_height <= visible_rows {
            self.scroll = 0;
            return;
        }

        let max_scroll = content_height.saturating_sub(visible_rows);
        if self.follow_latest {
            self.scroll = max_scroll;
        } else {
            self.scroll = self.scroll.min(max_scroll);
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
            }
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_down();
                }
            }
            KeyCode::Enter => {
                let command = self.input.trim().to_string();
                if command.eq_ignore_ascii_case("exit") {
                    self.should_exit = true;
                } else if command.eq_ignore_ascii_case("clear") {
                    self.reset_messages();
                } else if !command.is_empty() {
                    self.append_message(format!("> {command}"));
                    self.append_message(format!("Received: {command}"));
                }
                self.input.clear();
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(ch) => {
                self.input.push(ch);
            }
            _ => {}
        }
    }
}

fn render_ui(frame: &mut Frame, app: &mut App) {
    let outer = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(outer);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(chunks[0]);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "LOCALMIND ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("interactive shell"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::White));
    frame.render_widget(header, body[0]);

    let transcript: Vec<Line> = app
        .messages
        .iter()
        .map(|message| Line::from(Span::raw(message.clone())))
        .collect();

    app.sync_scroll(body[1].height);

    let messages = Paragraph::new(transcript)
        .block(Block::default().title("Session").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll as u16, 0));
    frame.render_widget(messages, body[1]);

    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            "> ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.input.as_str()),
    ]))
    .block(Block::default().title("Command").borders(Borders::ALL));
    frame.render_widget(input, chunks[1]);

    let cursor_x = chunks[1].x.saturating_add(2 + app.input.len() as u16);
    let cursor_y = chunks[1].y.saturating_add(1);
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    let result = loop {
        terminal.draw(|frame| render_ui(frame, &mut app))?;

        if app.should_exit {
            break Ok(());
        }

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            app.on_key(key);
        }
    };

    result
}