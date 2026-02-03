use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap,
    },
};
use serde::{Deserialize, Serialize};
use std::result;
use std::{
    fs::{self},
    ops::IndexMut,
};

const NORMAL_ROW_COLOR: Color = Color::Rgb(227, 227, 227);
const COMPLETED_ROW_COLOR: Color = Color::Rgb(100, 100, 100);
const TEXT_COLOR: Color = Color::Rgb(255, 255, 255);
const HIGHLIGHT_STYLE: Style = Style::new()
    .bg(Color::Rgb(60, 60, 60))
    .add_modifier(Modifier::BOLD);
const PATH: &str = "./tasks.json";

#[derive(Serialize, Deserialize, Debug, Default)]
struct Task {
    is_done: bool,
    description: String,
    sub_tasks: Vec<Task>,
}

#[derive(Debug, Default)]
struct AppState {
    tasks: Vec<Task>,
    list_state: ListState,
    new_task_added: bool,
    new_subtask_added: bool,
    input: String,
}

impl AppState {
    fn new() -> Self {
        let mut state = AppState::default();
        let result::Result::Ok(data) = fs::read_to_string(PATH) else {
            state.tasks = vec![];
            return state;
        };
        let tasks = serde_json::from_str(&data).unwrap_or_else(|_| vec![]);
        state.tasks = tasks;
        state
    }

    fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.tasks).unwrap();
        fs::write(path, json)
    }
}

enum FormAction {
    None,
    Submit,
    Escape,
}

fn main() -> Result<()> {
    let mut state = AppState::new();
    state.new_task_added = false;
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;

        if let Event::Key(key) = event::read()? {
            if app_state.new_task_added {
                match handle_new_task(key, app_state) {
                    FormAction::Escape => {
                        app_state.new_task_added = false;
                        app_state.input.clear();
                    }
                    FormAction::Submit => {
                        let new_task = Task {
                            is_done: false,
                            description: app_state.input.clone(),
                            sub_tasks: vec![],
                        };
                        app_state.new_task_added = false;
                        if app_state.new_subtask_added {
                            if let Some(index) = app_state.list_state.selected() {
                                let parent_task = app_state.tasks.index_mut(index);
                                parent_task.sub_tasks.push(new_task);
                                app_state.new_subtask_added = false;
                            }
                        } else {
                            app_state.tasks.push(new_task);
                        }
                        app_state.save(PATH)?;
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
        KeyCode::Char(c) => app_state.input.push(c),
        KeyCode::Backspace => {
            app_state.input.pop();
        }
        KeyCode::Esc => return FormAction::Escape,
        KeyCode::Enter => return FormAction::Submit,
        _ => {}
    }
    FormAction::None
}

fn handle_key(key: KeyEvent, app_state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return true,
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(index) = app_state.list_state.selected() {
                if let Some(task) = app_state.tasks.get_mut(index) {
                    task.is_done = !task.is_done;
                    let _ = app_state.save(PATH);
                }
            }
        }
        KeyCode::Char('k') => app_state.list_state.select_previous(),
        KeyCode::Char('j') => app_state.list_state.select_next(),
        KeyCode::Char('a') => app_state.new_task_added = true,
        KeyCode::Char('d') => {
            if let Some(index) = app_state.list_state.selected() {
                app_state.tasks.remove(index);
                let _ = app_state.save(PATH);
            }
        }
        KeyCode::Char('A') => {
            app_state.new_task_added = true;
            app_state.new_subtask_added = true;
        }
        KeyCode::Char('J') => {
            //TODO: expand subtasks
        }
        KeyCode::Char('K') => {
            //TODO: collapse subtasks
        }
        _ => {}
    }
    false
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);

    let [main_area, footer_area] = vertical.areas(frame.area());

    if app_state.tasks.is_empty() {
        let empty_msg = Paragraph::new("No tasks yet.\nPress 'a' to add one.")
            .centered()
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, main_area);
    } else {
        let items: Vec<ListItem> = app_state
            .tasks
            .iter()
            .map(|task| {
                let (icon, style) = if task.is_done {
                    ("", Style::default().fg(COMPLETED_ROW_COLOR))
                } else {
                    ("", Style::default().fg(NORMAL_ROW_COLOR))
                };

                let desc_style = if task.is_done {
                    Style::default()
                        .fg(COMPLETED_ROW_COLOR)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(TEXT_COLOR)
                };

                let dropdown = if task.sub_tasks.is_empty() {
                    " "
                } else {
                    ""
                };

                let line = Line::from(vec![
                    Span::styled(dropdown, Style::default().fg(Color::Gray)),
                    Span::styled(format!(" {} ", icon), style),
                    Span::styled(&task.description, desc_style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Tasks ")
            .title_style(Style::default().fg(Color::Yellow))
            .padding(Padding::horizontal(1));

        let list = List::new(items)
            .block(list_block)
            .highlight_style(HIGHLIGHT_STYLE)
            .highlight_symbol("▎");

        frame.render_stateful_widget(list, main_area, &mut app_state.list_state);
    }

    let help_text = Line::from(vec![
        Span::styled(
            "h - ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("help | "),
        Span::styled(
            "q/esc - ",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("quit"),
    ]);

    let footer = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, footer_area);

    if app_state.new_task_added {
        let popup_area = center_area(frame.area(), 60, 20);

        frame.render_widget(Clear, popup_area);

        let popup_block = Block::bordered()
            .title(" Create New Task ")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let input_text = Paragraph::new(app_state.input.as_str())
            .wrap(Wrap { trim: true })
            .block(popup_block);

        frame.render_widget(input_text, popup_area);

        let hint_area = Rect::new(
            popup_area.x,
            popup_area.y + popup_area.height,
            popup_area.width,
            1,
        );
        frame.render_widget(
            Paragraph::new("Enter: Submit | Esc: Cancel")
                .centered()
                .fg(Color::DarkGray),
            hint_area,
        );
    }
}

fn center_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
