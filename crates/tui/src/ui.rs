use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;

pub fn render_ui(frame: &mut Frame, app: &mut App) {
    let outer = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(outer);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(chunks[0]);

    render_header(frame, body[0], &app.config.summary());
    render_messages(frame, app, body[1]);
    render_input(frame, app, chunks[1]);
}

fn render_header(frame: &mut Frame, area: Rect, summary: &str) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "LOCALMIND ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("interactive shell · "),
        Span::styled(summary.to_string(), Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::White));

    frame.render_widget(header, area);
}

fn render_messages(frame: &mut Frame, app: &mut App, area: Rect) {
    app.sync_scroll(area.height);

    let transcript: Vec<Line> = app
        .messages
        .iter()
        .map(|message| Line::from(Span::raw(message.clone())))
        .collect();

    let messages = Paragraph::new(transcript)
        .block(Block::default().title("Session").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll as u16, 0));

    frame.render_widget(messages, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(Line::from(vec![
        Span::styled(
            "> ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.input.as_str()),
    ]))
    .block(Block::default().title("Command").borders(Borders::ALL));

    frame.render_widget(input, area);

    let cursor_x = area.x.saturating_add(2 + app.input.len() as u16);
    let cursor_y = area.y.saturating_add(1);
    frame.set_cursor_position((cursor_x, cursor_y));
}