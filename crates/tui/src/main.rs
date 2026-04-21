use ratatui::backend::TestBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

fn render_ui(frame: &mut Frame, response: &str) {
    let area = frame.area();
    let block = Block::default().title("LOCALMIND").borders(Borders::ALL);
    let paragraph = Paragraph::new(response.to_string()).block(block);
    frame.render_widget(paragraph, area);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = "LOCALMIND skeleton";

    let backend = TestBackend::new(80, 8);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_ui(frame, response))?;

    println!("{response}");
    Ok(())
}
