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

#[derive(Serialize, Deserialize, Debug, Default)]
struct Folder {
    name: String,
    tasks: Vec<Task>,
    is_draft: bool,
}

impl Folder {
    fn new(name: String) -> Self {
        Self {
            name,
            tasks: vec![],
            is_draft: false,
        }
    }

    fn new_draft() -> Self {
        Self {
            name: String::new(),
            tasks: vec![],
            is_draft: true,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
enum Focus {
    #[default]
    Folders,
    Tasks,
}

#[derive(Debug, Default)]
struct AppState {
    folders: Vec<Folder>,
    folder_state: ListState,
    task_state: ListState,
    new_item_added: bool,
    focus: Focus,
}

impl AppState {
    fn new() -> Self {
        let mut state = AppState::default();
        let result::Result::Ok(data) = fs::read_to_string(PATH) else {
            state.folders = vec![Folder::new("General".to_string())];
            state.folder_state.select(Some(0));
            return state;
        };
        state.folders = serde_json::from_str(&data)
            .unwrap_or_else(|_| vec![Folder::new("General".to_string())]);
        state
    }

    fn save(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.folders).unwrap();
        fs::write(path, json)
    }

    fn get_active_folder_mut(&mut self) -> Option<&mut Folder> {
        let idx = self.folder_state.selected()?;
        self.folders.get_mut(idx)
    }
}

struct FlatItem<'a> {
    task: &'a Task,
    depth: usize,
    index_path: Vec<usize>,
}

fn main() -> Result<()> {
    let mut state = AppState::new();
    state.new_item_added = false;
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
            if app_state.new_item_added {
                handle_new_item(key, app_state)?
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
    let current_index = app_state.task_state.selected().unwrap_or(0);
    let folder = app_state.get_active_folder_mut()?;
    let flat_view = flatten_tasks(&folder.tasks, 0, &[]);
    flat_view
        .get(current_index)
        .map(|item| item.index_path.clone())
}

fn handle_new_item(key: KeyEvent, app_state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Char(c) => {
            if app_state.focus == Focus::Folders {
                if let Some(draft) = app_state.folders.iter_mut().find(|f| f.is_draft) {
                    draft.name.push(c);
                }
            } else if let Some(folder) = app_state.get_active_folder_mut() {
                if let Some(draft) = find_draft_mut(&mut folder.tasks) {
                    draft.description.push(c);
                }
            }
        }
        KeyCode::Backspace => {
            if app_state.focus == Focus::Folders {
                if let Some(draft) = app_state.folders.iter_mut().find(|f| f.is_draft) {
                    draft.name.pop();
                }
            } else if let Some(folder) = app_state.get_active_folder_mut() {
                if let Some(draft) = find_draft_mut(&mut folder.tasks) {
                    draft.description.pop();
                }
            }
        }
        KeyCode::Esc => {
            remove_draft(app_state);
            app_state.new_item_added = false;
        }
        KeyCode::Enter => {
            if app_state.focus == Focus::Folders {
                if let Some(draft) = app_state.folders.iter_mut().find(|f| f.is_draft) {
                    if draft.name.trim().is_empty() {
                        remove_draft(app_state);
                    } else {
                        draft.is_draft = false;
                    }
                }
            } else if let Some(folder) = app_state.get_active_folder_mut() {
                if let Some(draft) = find_draft_mut(&mut folder.tasks) {
                    if draft.description.trim().is_empty() {
                        remove_draft(app_state);
                    } else {
                        draft.is_draft = false;
                    }
                }
            }
            app_state.new_item_added = false;
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
    if let Some(folder) = app_state.get_active_folder_mut() {
        let flat_tasks = flatten_tasks(&folder.tasks, 0, &[]);
        if let Some(idx) = flat_tasks.iter().position(|item| item.task.is_draft) {
            app_state.task_state.select(Some(idx));
        }
    }
}

fn remove_draft(app_state: &mut AppState) {
    app_state.folders.retain(|f| !f.is_draft);

    if let Some(folder) = app_state.get_active_folder_mut() {
        remove_task_draft(&mut folder.tasks);
    }
}

fn remove_task_draft(tasks: &mut Vec<Task>) -> bool {
    let mut to_remove = None;
    for (i, task) in tasks.iter_mut().enumerate() {
        if task.is_draft {
            to_remove = Some(i);
            break;
        }
        if remove_task_draft(&mut task.sub_tasks) {
            return true;
        }
    }
    if let Some(i) = to_remove {
        tasks.remove(i);
        return true;
    }
    false
}

fn update_parent_completion(tasks: &mut Vec<Task>, mut path: Vec<usize>) {
    while path.len() > 1 {
        path.pop();
        if let Some(parent) = get_task_by_path(tasks, &path) {
            parent.is_done = parent.sub_tasks.iter().all(|t| t.is_done);
        }
    }
}

fn get_parent_path(path: Vec<usize>) -> Option<Vec<usize>> {
    if path.len() > 1 {
        // has a parent
        let parent_path = path[0..path.len() - 1].to_vec();

        return Some(parent_path);
    }

    None
}

fn handle_key(key: KeyEvent, app_state: &mut AppState) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => return true,
        KeyCode::Tab => {
            app_state.focus = if app_state.focus == Focus::Folders {
                Focus::Tasks
            } else {
                Focus::Folders
            };
        }
        KeyCode::Char(' ') => {
            if app_state.focus == Focus::Tasks {
                if let Some(path) = get_selected_path(app_state) {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        if let Some(task) = get_task_by_path(&mut folder.tasks, &path) {
                            if !task.sub_tasks.is_empty() {
                                return false;
                            }
                            task.is_done = !task.is_done;

                            update_parent_completion(&mut folder.tasks, path);

                            let _ = app_state.save(PATH); // TODO: handle if an error is returned here
                        }
                    }
                }
            }
        }
        KeyCode::Char('k') => match app_state.focus {
            Focus::Folders => app_state.folder_state.select_previous(),
            Focus::Tasks => app_state.task_state.select_previous(),
        },
        KeyCode::Char('j') => match app_state.focus {
            Focus::Folders => app_state.folder_state.select_next(),
            Focus::Tasks => app_state.task_state.select_next(),
        },
        KeyCode::Char('a') => {
            if app_state.focus == Focus::Folders {
                app_state.folders.push(Folder::new_draft());
                app_state
                    .folder_state
                    .select(Some(app_state.folders.len() - 1));
            } else {
                let path_opt = get_selected_path(app_state);

                if let Some(folder_idx) = app_state.folder_state.selected() {
                    let folder = &mut app_state.folders[folder_idx];
                    let new_draft_task = Task::new_draft();

                    if let Some(path) = path_opt {
                        if let Some(parent_path) = get_parent_path(path) {
                            if let Some(parent_task) =
                                get_task_by_path(&mut folder.tasks, &parent_path)
                            {
                                parent_task.sub_tasks.push(new_draft_task);
                            }
                        } else {
                            folder.tasks.push(new_draft_task);
                        }
                    } else {
                        folder.tasks.push(new_draft_task);
                    }
                }

                app_state.new_item_added = true;
                jump_selection_to_draft(app_state);
            }
        }
        KeyCode::Char('d') => {
            if app_state.focus == Focus::Folders {
                if let Some(idx) = app_state.folder_state.selected() {
                    app_state.folders.remove(idx);
                    if app_state.folders.is_empty() {
                        app_state.folder_state.select(None);
                    } else if idx >= app_state.folders.len() {
                        app_state
                            .folder_state
                            .select(Some(app_state.folders.len() - 1));
                    }
                    let _ = app_state.save(PATH);
                }
            } else {
                if let Some(path) = get_selected_path(app_state) {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        if path.len() == 1 {
                            folder.tasks.remove(path[0]);
                        } else {
                            let parent_path = &path[0..path.len() - 1];
                            let child_idx = path[path.len() - 1];
                            if let Some(parent) = get_task_by_path(&mut folder.tasks, parent_path) {
                                parent.sub_tasks.remove(child_idx);
                            }
                        }
                        let _ = app_state.save(PATH); // TODO: handle if error is returned here

                        // let flat_view = flatten_tasks(&folder.tasks, 0, &[]);
                        // let current_index = app_state.task_state.selected().unwrap_or(0);
                        // // current index correction
                        // if current_index >= flat_view.len().saturating_sub(1) {
                        //     app_state
                        //         .task_state
                        //         .select(Some(flat_view.len().saturating_sub(2)));
                        // }
                    }
                }
            }
        }
        KeyCode::Char('A') => {
            // add a sub-task
            if app_state.focus == Focus::Tasks {
                let new_draft_task = Task::new_draft();

                if let Some(path) = get_selected_path(app_state) {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        if let Some(parent_task) = get_task_by_path(&mut folder.tasks, &path) {
                            parent_task.sub_tasks.push(new_draft_task);
                            parent_task.expanded = true;
                            app_state.new_item_added = true;
                            jump_selection_to_draft(app_state);
                        }
                    }
                }
            }
        }
        KeyCode::Enter => {
            // expand a task
            if app_state.focus == Focus::Tasks {
                if let Some(path) = get_selected_path(app_state) {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        if let Some(task) = get_task_by_path(&mut folder.tasks, &path) {
                            if !task.sub_tasks.is_empty() {
                                task.expanded = !task.expanded;
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Char('p') => {
            // go to parent task
            if app_state.focus == Focus::Tasks {
                let path_opt = get_selected_path(app_state);
                if let Some(parent_path) = path_opt {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        let flat_tasks = flatten_tasks(&folder.tasks, 0, &[]);
                        if let Some(idx) = flat_tasks
                            .iter()
                            .position(|item| item.index_path == parent_path)
                        {
                            app_state.task_state.select(Some(idx));
                        }
                    }
                }
            }
        }
        KeyCode::Char('P') => {
            // go to root parent
            if app_state.focus == Focus::Tasks {
                if let Some(path) = get_selected_path(app_state) {
                    if let Some(folder) = app_state.get_active_folder_mut() {
                        if path.len() > 1 {
                            let root = path[0..1].to_vec();

                            let flat_tasks = flatten_tasks(&folder.tasks, 0, &[]);
                            if let Some(idx) =
                                flat_tasks.iter().position(|item| item.index_path == root)
                            {
                                app_state.task_state.select(Some(idx));
                            }
                        }
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

    let horizontal = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]);
    let [folder_area, task_area] = horizontal.areas(main_area);

    // --- Render Folders ---
    let folder_items: Vec<ListItem> = app_state
        .folders
        .iter()
        .map(|f| {
            if f.is_draft {
                ListItem::new(Line::from(vec![
                    Span::styled(" > ", Style::default().fg(Color::Yellow)),
                    Span::styled(&f.name, Style::default().fg(Color::Yellow)),
                    Span::styled("█", Style::default().fg(Color::White)),
                ]))
            } else {
                ListItem::new(Line::from(vec![Span::raw(format!(" 󰉋 {}", f.name))]))
            }
        })
        .collect();

    let folder_border_color = if app_state.focus == Focus::Folders {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let folder_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(folder_border_color))
        .title(" Folders ");

    let folder_list = List::new(folder_items)
        .block(folder_block)
        .highlight_style(HIGHLIGHT_STYLE);

    frame.render_stateful_widget(folder_list, folder_area, &mut app_state.folder_state);

    // --Render Tasks--
    let task_border_color = if app_state.focus == Focus::Tasks {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let task_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(task_border_color))
        .title(" Tasks ")
        .padding(Padding::horizontal(1));

    if let Some(folder_idx) = app_state.folder_state.selected() {
        if let Some(folder) = app_state.folders.get(folder_idx) {
            let flat_tasks = flatten_tasks(&folder.tasks, 0, &[]);

            if flat_tasks.is_empty() {
                let empty_msg = Paragraph::new("No tasks yet.\nPress 'a' to add one.")
                    .centered()
                    .block(task_block)
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty_msg, task_area);
            } else {
                let items: Vec<ListItem> = flat_tasks
                    .iter()
                    .map(|item| {
                        let task = item.task;
                        let indent = "   ".repeat(item.depth);

                        if task.is_draft {
                            let line = Line::from(vec![
                                Span::raw(indent),
                                Span::styled(
                                    " > ",
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                                ),
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

                        let mut spans = vec![
                            Span::styled(indent, Style::default()),
                            Span::styled(format!(" {} ", icon), style),
                            Span::styled(&task.description, desc_style),
                        ];

                        if !task.sub_tasks.is_empty() {
                            let mut done_count: usize = 0;
                            task.sub_tasks.iter().for_each(|item| {
                                if item.is_done {
                                    done_count = done_count + 1;
                                }
                            });
                            spans.push(Span::styled(
                                format!("  ({}/{})", done_count, task.sub_tasks.len()),
                                Style::default().fg(Color::DarkGray),
                            ));
                        }

                        ListItem::new(Line::from(spans))
                    })
                    .collect();

                let list = List::new(items)
                    .block(task_block)
                    .highlight_style(HIGHLIGHT_STYLE);

                frame.render_stateful_widget(list, task_area, &mut app_state.task_state);
            }
        }
    } else {
        // No folder selected
        frame.render_widget(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(task_border_color)),
            task_area,
        );
    }

    // --Footer--
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
