use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use localmind_config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    None,
    Exit,
    Clear,
    Help,
    ConfigHelp,
    ConfigView,
    ConfigEdit,
    ConfigReset,
}

pub struct App {
    pub config: Config,
    initial_message: String,
    pub messages: Vec<String>,
    pub input: String,
    pub scroll: usize,
    follow_latest: bool,
    pub should_exit: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        let initial_message =
            "Type ? for help, config for config commands, clear to reset, or exit to quit."
                .to_string();

        Self {
            config,
            messages: vec![initial_message.clone()],
            initial_message,
            input: String::new(),
            scroll: 0,
            follow_latest: true,
            should_exit: false,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> CommandAction {
        if key.kind != KeyEventKind::Press {
            return CommandAction::None;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
                CommandAction::Exit
            }
            KeyCode::Up => {
                self.scroll_up();
                CommandAction::None
            }
            KeyCode::Down => {
                self.scroll_down();
                CommandAction::None
            }
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_up();
                }
                CommandAction::None
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_down();
                }
                CommandAction::None
            }
            KeyCode::Enter => {
                let command = self.input.trim().to_string();

                let action = match command.as_str() {
                    "?" | "help" => CommandAction::Help,
                    "config" => CommandAction::ConfigHelp,
                    "config view" => CommandAction::ConfigView,
                    "config edit" => CommandAction::ConfigEdit,
                    "config reset" => CommandAction::ConfigReset,
                    _ if command.eq_ignore_ascii_case("exit") => CommandAction::Exit,
                    _ if command.eq_ignore_ascii_case("clear") => CommandAction::Clear,
                    _ if command.starts_with("config ") => {
                        self.append_message("Unknown config command. Use: config view, config edit, config reset".to_string());
                        CommandAction::None
                    }
                    _ if !command.is_empty() => {
                        self.append_message(format!("> {command}"));
                        self.append_message(format!("Received: {command}"));
                        CommandAction::None
                    }
                    _ => CommandAction::None,
                };

                if matches!(action, CommandAction::Exit) {
                    self.should_exit = true;
                } else if matches!(action, CommandAction::Clear) {
                    self.reset_messages();
                }

                self.input.clear();
                action
            }
            KeyCode::Backspace => {
                self.input.pop();
                CommandAction::None
            }
            KeyCode::Char(ch) => {
                self.input.push(ch);
                CommandAction::None
            }
            _ => CommandAction::None,
        }
    }

    pub fn push_system_message(&mut self, message: impl Into<String>) {
        self.append_message(message.into());
    }

    pub fn sync_scroll(&mut self, viewport_height: u16) {
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
}