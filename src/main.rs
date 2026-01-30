use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::ToSpan,
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
};

#[derive(Debug, Default)]
struct AppState {
    items: Vec<Task>,
    list_state: ListState,
    new_item_added: bool,
    input: String,
}

#[derive(Debug, Default)]
struct Task {
    is_done: bool,
    description: String,
}

enum FormAction {
    None,
    Submit,
    Escape,
}

fn main() -> Result<()> {
    let mut state = AppState::default();
    state.new_item_added = false;
    color_eyre::install()?;

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
            if app_state.new_item_added {
                match handle_new_task(key, app_state) {
                    FormAction::Escape => {
                        app_state.new_item_added = false;
                        app_state.input.clear();
                    }
                    FormAction::Submit => {
                        app_state.new_item_added = false;
                        app_state.items.push(Task {
                            is_done: false,
                            description: app_state.input.clone(),
                        });
                        app_state.input.clear();
                    }
                    FormAction::None => {}
                }
            } else {
                if handle_key(key, app_state) {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn handle_new_task(key: KeyEvent, app_state: &mut AppState) -> FormAction {
    match key.code {
        event::KeyCode::Char(c) => {
            app_state.input.push(c);
        }
        event::KeyCode::Backspace => {
            app_state.input.pop();
        }
        event::KeyCode::Esc => {
            return FormAction::Escape;
        }
        event::KeyCode::Enter => {
            return FormAction::Submit;
        }
        _ => {}
    }
    FormAction::None
}

fn handle_key(key: KeyEvent, app_state: &mut AppState) -> bool {
    match key.code {
        event::KeyCode::Esc => {
            return true;
        }
        event::KeyCode::Enter => {
            if let Some(index) = app_state.list_state.selected() {
                if let Some(task) = app_state.items.get_mut(index) {
                    task.is_done = !task.is_done;
                }
            }
        }
        event::KeyCode::Char(char) => match char {
            'k' => {
                app_state.list_state.select_previous();
            }
            'j' => {
                app_state.list_state.select_next();
            }
            'a' => {
                app_state.new_item_added = true;
            }
            'd' => {
                if let Some(index) = app_state.list_state.selected() {
                    app_state.items.remove(index);
                }
            }
            _ => {}
        },
        _ => {}
    }
    false
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let [border_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(frame.area());

    let [inner_area] = Layout::vertical([Constraint::Fill(1)])
        .margin(1)
        .areas(border_area);

    let [input_area] = Layout::vertical([Constraint::Fill(1)]).areas(Rect::new(
        inner_area.x,
        inner_area.y + inner_area.height - 3,
        inner_area.width,
        3,
    ));

    Block::bordered()
        .border_type(ratatui::widgets::BorderType::Rounded)
        .render(border_area, frame.buffer_mut());

    let list = List::new(app_state.items.iter().map(|x| {
        let value = if x.is_done {
            x.description.to_span().crossed_out()
        } else {
            x.description.to_span()
        };
        ListItem::from(value)
    }))
    .highlight_symbol(">")
    .highlight_style(Style::default().fg(Color::Red));

    frame.render_stateful_widget(list, inner_area, &mut app_state.list_state);

    if app_state.new_item_added {
        Paragraph::new(app_state.input.as_str())
            .block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title("create a new task".to_span().into_centered_line()),
            )
            .render(input_area, frame.buffer_mut());
    }
}
