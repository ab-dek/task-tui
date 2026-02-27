use color_eyre::eyre::{Ok, Result};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, ListState, Padding, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::result;

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
    expanded: bool, // to display subtasks
    is_draft: bool,
}

impl Task {
    fn new_draft() -> Self {
        Self {
            is_done: false,
            description: String::new(),
            sub_tasks: vec![],
            expanded: false,
            is_draft: true,
        }
    }
}
// fn flatten_tasks() {}

#[derive(Debug, Default)]
struct AppState {
    tasks: Vec<Task>,
    list_state: ListState,
    new_task_added: bool,
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

struct FlatItem<'a> {
    task: &'a Task,
    depth: usize,
    index_path: Vec<usize>,
}

fn main() -> Result<()> {
    let mut state = AppState::new();
    state.new_task_added = false;
    color_eyre::install()?;

    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);

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
                handle_new_task(key, app_state)?
            } else {
                if handle_key(key, app_state) {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn flatten_tasks<'a>(tasks: &'a [Task], depth: usize, current_path: &[usize]) -> Vec<FlatItem<'a>> {
    let mut flat_list = Vec::new();
    for (i, task) in tasks.iter().enumerate() {
        let mut new_path = current_path.to_vec();
        new_path.push(i);

        flat_list.push(FlatItem {
            task,
            depth,
            index_path: new_path.clone(),
        });

        if task.expanded {
            let children = flatten_tasks(&task.sub_tasks, depth + 1, &new_path);
            flat_list.extend(children);
        }
    }

    flat_list
}

fn get_task_by_path<'a>(tasks: &'a mut Vec<Task>, path: &[usize]) -> Option<&'a mut Task> {
    if path.is_empty() {
        return None;
    }
    let mut current = tasks.get_mut(path[0])?;
    for &idx in &path[1..] {
        current = current.sub_tasks.get_mut(idx)?;
    }
    Some(current)
}

fn get_selected_path(app_state: &mut AppState) -> Option<Vec<usize>> {
    let flat_view = flatten_tasks(&app_state.tasks, 0, &[]);
    let current_index = app_state.list_state.selected().unwrap_or(0);
    flat_view
        .get(current_index)
        .map(|item| item.index_path.clone())
}

fn handle_new_task(key: KeyEvent, app_state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            if let Some(draft) = find_draft_mut(&mut app_state.tasks) {
                draft.description.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(draft) = find_draft_mut(&mut app_state.tasks) {
                draft.description.pop();
            }
        }
        KeyCode::Esc => {
            remove_draft(&mut app_state.tasks);
            app_state.new_task_added = false;
        }
        KeyCode::Enter => {
            if let Some(draft) = find_draft_mut(&mut app_state.tasks) {
                if draft.description.trim().is_empty() {
                    remove_draft(&mut app_state.tasks);
                } else {
                    draft.is_draft = false;
                }
            }
            app_state.new_task_added = false;
            app_state.save(PATH)?
        }
        _ => {}
    }
    Ok(())
}

fn find_draft_mut(tasks: &mut Vec<Task>) -> Option<&mut Task> {
    for task in tasks.iter_mut() {
        if task.is_draft {
            return Some(task);
        }
        if let Some(draft) = find_draft_mut(&mut task.sub_tasks) {
            return Some(draft);
        }
    }
    None
}

fn jump_selection_to_draft(app_state: &mut AppState) {
    let flat_tasks = flatten_tasks(&app_state.tasks, 0, &[]);
    if let Some(idx) = flat_tasks.iter().position(|item| item.task.is_draft) {
        app_state.list_state.select(Some(idx));
    }
}

fn remove_draft(tasks: &mut Vec<Task>) -> bool {
    let mut to_remove = None;
    for (i, task) in tasks.iter_mut().enumerate() {
        if task.is_draft {
            to_remove = Some(i);
            break;
        }
        if remove_draft(&mut task.sub_tasks) {
            return true;
        }
    }
    if let Some(i) = to_remove {
        tasks.remove(i);
        return true;
    }
    false
}

fn handle_key(key: KeyEvent, app_state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return true,
        KeyCode::Char(' ') => {
            if let Some(path) = get_selected_path(app_state) {
                if let Some(task) = get_task_by_path(&mut app_state.tasks, &path) {
                    task.is_done = !task.is_done;
                    let _ = app_state.save(PATH);
                }
            }
        }
        KeyCode::Char('k') => app_state.list_state.select_previous(),
        KeyCode::Char('j') => app_state.list_state.select_next(),
        KeyCode::Char('a') => {
            let new_draft_task = Task::new_draft();
            if let Some(path) = get_selected_path(app_state) {
                if path.len() > 1 {
                    // has a parent
                    let parent_path = path[0..path.len() - 1].to_vec();
                    if let Some(parent_task) = get_task_by_path(&mut app_state.tasks, &parent_path)
                    {
                        parent_task.sub_tasks.push(new_draft_task);
                    }
                } else {
                    app_state.tasks.push(new_draft_task);
                }
            } else {
                app_state.tasks.push(new_draft_task);
            }
            app_state.new_task_added = true;
            jump_selection_to_draft(app_state);
        }
        KeyCode::Char('d') => {
            if let Some(path) = get_selected_path(app_state) {
                if path.len() == 1 {
                    app_state.tasks.remove(path[0]);
                    let _ = app_state.save(PATH);
                } else {
                    let parent_path = &path[0..path.len() - 1];
                    let child_idx = path[path.len() - 1];
                    if let Some(parent) = get_task_by_path(&mut app_state.tasks, parent_path) {
                        parent.sub_tasks.remove(child_idx);
                    }
                }

                let flat_view = flatten_tasks(&app_state.tasks, 0, &[]);
                let current_index = app_state.list_state.selected().unwrap_or(0);
                // current index correction
                if current_index >= flat_view.len().saturating_sub(1) {
                    app_state
                        .list_state
                        .select(Some(flat_view.len().saturating_sub(2)));
                }
            }
        }
        KeyCode::Char('A') => {
            let new_draft_task = Task::new_draft();

            if let Some(path) = get_selected_path(app_state) {
                if let Some(parent_task) = get_task_by_path(&mut app_state.tasks, &path) {
                    parent_task.sub_tasks.push(new_draft_task);
                    parent_task.expanded = true;
                    app_state.new_task_added = true;
                    jump_selection_to_draft(app_state);
                }
            }
        }
        KeyCode::Enter => {
            if let Some(path) = get_selected_path(app_state) {
                if let Some(task) = get_task_by_path(&mut app_state.tasks, &path) {
                    if !task.sub_tasks.is_empty() {
                        task.expanded = !task.expanded;
                    }
                }
            }
        }
        _ => {}
    }
    false
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    // let chunks = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
    //     .split(frame.area());

    let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);

    let [main_area, footer_area] = vertical.areas(frame.area());

    let flat_tasks = flatten_tasks(&app_state.tasks, 0, &[]);

    if flat_tasks.is_empty() {
        let empty_msg = Paragraph::new("No tasks yet.\nPress 'a' to add one.")
            .centered()
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, main_area);
    } else {
        let items: Vec<ListItem> = flat_tasks
            .iter()
            .map(|item| {
                let task = item.task;
                let indent = "   ".repeat(item.depth);

                if task.is_draft {
                    let line = Line::from(vec![
                        Span::raw(indent),
                        Span::styled(&task.description, Style::default().fg(Color::Yellow)),
                        Span::styled("█", Style::default().fg(Color::White)),
                    ]);
                    return ListItem::new(line);
                }

                let (icon, style) = if task.sub_tasks.is_empty() {
                    if task.is_done {
                        ("", Style::default().fg(COMPLETED_ROW_COLOR))
                    } else {
                        ("", Style::default().fg(NORMAL_ROW_COLOR))
                    }
                } else {
                    if task.expanded {
                        ("", Style::default().fg(NORMAL_ROW_COLOR))
                    } else {
                        ("", Style::default().fg(NORMAL_ROW_COLOR))
                    }
                };

                let desc_style = if task.is_done {
                    Style::default()
                        .fg(COMPLETED_ROW_COLOR)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(TEXT_COLOR)
                };

                if !task.sub_tasks.is_empty() {} //TODO: show task progress

                let line = Line::from(vec![
                    Span::styled(indent, Style::default()),
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
            .highlight_style(HIGHLIGHT_STYLE);

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

    // let log_widget = TuiLoggerWidget::default()
    //     .block(Block::default().title("Logs").borders(Borders::ALL))
    //     .style_error(Style::default().fg(Color::Red))
    //     .style_warn(Style::default().fg(Color::Yellow))
    //     .style_info(Style::default().fg(Color::Cyan));

    // frame.render_widget(log_widget, chunks[1]);
}
