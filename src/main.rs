use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, ListState, Widget},
};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Task>,
    list_state: ListState,
}

#[derive(Debug, Default)]
struct Task {
    is_done: bool,
    description: String,
}

fn main() -> Result<()> {
    let mut state = AppState::default();
    color_eyre::install()?;

    state.items.push(Task {
        is_done: false,
        description: String::from("Finish app 1"),
    });
    state.items.push(Task {
        is_done: false,
        description: String::from("Finish app 2 "),
    });
    state.items.push(Task {
        is_done: false,
        description: String::from("Finish app 3"),
    });

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        // render
        terminal.draw(|f| render(f, app_state))?;
        // input
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                event::KeyCode::Char(char) => match char {
                    'D' => {
                        if let Some(index) = app_state.list_state.selected() {
                            app_state.items.remove(index);
                        }
                    }
                    'k' => {
                        app_state.list_state.select_previous();
                    }
                    'j' => {
                        app_state.list_state.select_next();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [inner_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .fg(Color::Yellow)
        .render(border_area, frame.buffer_mut());

    let list = List::new(
        app_state
            .items
            .iter()
            .map(|x| ListItem::from(x.description.clone())),
    )
    .highlight_symbol(">")
    .highlight_style(Style::default().fg(Color::Red));

    frame.render_stateful_widget(list, inner_area, &mut app_state.list_state);
}
