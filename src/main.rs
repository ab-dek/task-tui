use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Task>,
}

#[derive(Debug, Default)]
struct Task {
    is_done: bool,
    description: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);

    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        // render
        terminal.draw(render)?;
        // input
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame) {
    Paragraph::new("Hello Tui").render(frame.area(), frame.buffer_mut());
}
