use std::env;
use std::process;
use std::time::Duration;

use crossterm::event::{self, Event};
mod app;
mod terminal;
mod ui;

use app::App;
use app::CommandAction;
use localmind_config::Config;
use terminal::TerminalSession;
use ui::render_ui;

enum CliCommand {
    Run,
    ConfigView,
    ConfigEdit,
    ConfigReset,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match parse_command() {
        CliCommand::Run => run_tui()?,
        CliCommand::ConfigView => view_config()?,
        CliCommand::ConfigEdit => edit_config()?,
        CliCommand::ConfigReset => reset_config()?,
    }

    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_or_setup_interactive()?;
    let mut session = TerminalSession::new()?;
    let mut app = App::new(config);

    let result = loop {
        session.terminal_mut().draw(|frame| render_ui(frame, &mut app))?;

        if app.should_exit {
            break Ok(());
        }

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            match app.on_key(key) {
                CommandAction::None => {}
                CommandAction::Exit => app.should_exit = true,
                CommandAction::Clear => app.push_system_message("Session cleared."),
                CommandAction::Help => show_help(&mut app),
                CommandAction::ConfigHelp => show_config_help(&mut app),
                CommandAction::ConfigView => show_config_view(&mut app)?,
                CommandAction::ConfigEdit => {
                    drop(session);
                    let updated = edit_config_interactive()?;
                    app.config = updated;
                    app.push_system_message("Configuration updated.");
                    session = TerminalSession::new()?;
                }
                CommandAction::ConfigReset => {
                    drop(session);
                    reset_config_interactive()?;
                    app.config = Config::load_or_setup_interactive()?;
                    app.push_system_message("Configuration reset.");
                    session = TerminalSession::new()?;
                }
            }
        }
    };

    result
}

fn show_help(app: &mut App) {
    app.push_system_message("Available commands:");
    app.push_system_message("? or help - show this help");
    app.push_system_message("clear - clear the session messages");
    app.push_system_message("config - show config commands");
    app.push_system_message("exit - quit the application");
}

fn show_config_help(app: &mut App) {
    app.push_system_message("Config commands:");
    app.push_system_message("config view - show the stored config");
    app.push_system_message("config edit - edit the stored config");
    app.push_system_message("config reset - delete config and re-run setup");
}

fn show_config_view(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    for line in config.view_string()?.lines() {
        app.push_system_message(line.to_string());
    }
    Ok(())
}

fn edit_config_interactive() -> Result<Config, Box<dyn std::error::Error>> {
    Ok(Config::edit_interactive()?)
}

fn reset_config_interactive() -> Result<(), Box<dyn std::error::Error>> {
    Config::reset()?;
    Ok(())
}

fn view_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    print!("{}", config.view_string()?);
    Ok(())
}

fn edit_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::edit_interactive()?;
    println!("Saved configuration for {}", config.summary());
    Ok(())
}

fn reset_config() -> Result<(), Box<dyn std::error::Error>> {
    Config::reset()?;
    println!("Configuration removed. The next run will prompt setup.");
    Ok(())
}

fn parse_command() -> CliCommand {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        None => CliCommand::Run,
        Some("config") => match args.next().as_deref() {
            Some("view") => CliCommand::ConfigView,
            Some("edit") => CliCommand::ConfigEdit,
            Some("reset") => CliCommand::ConfigReset,
            Some(other) => usage_and_exit(&format!("Unknown config command: {other}")),
            None => usage_and_exit("Missing config command."),
        },
        Some(other) => usage_and_exit(&format!("Unknown command: {other}")),
    }
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("{message}");
    eprintln!("Usage: tui [config view|edit|reset]");
    process::exit(2);
}