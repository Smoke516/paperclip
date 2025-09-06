use crate::app::{App, AppMode, ViewMode};
use crate::todo::TodoStatus;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let _colors = &app.colors;
    
    if app.show_help {
        draw_help(f, app);
        return;
    }
    
    // Check for popup modes
    if matches!(app.mode, AppMode::TagSelection | AppMode::ContextSelection | AppMode::TemplateSelection | AppMode::RecurrenceSelection | AppMode::WorkspaceSelection) {
        draw_main_ui(f, app);
        draw_selection_popup(f, app);
        return;
    }
    
    // Check for notes editing or viewing mode
    if matches!(app.mode, AppMode::EditNotes | AppMode::ViewNotes) {
        draw_main_ui(f, app);
        if matches!(app.mode, AppMode::EditNotes) {
            draw_notes_editor(f, app);
        } else {
            draw_notes_viewer(f, app);
        }
        return;
    }
    
    // Check for create workspace mode
    if matches!(app.mode, AppMode::CreateWorkspace) {
        draw_create_workspace_ui(f, app);
        return;
    }

    draw_main_ui(f, app);
}

fn draw_main_ui(f: &mut Frame, app: &mut App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
            Constraint::Length(if matches!(app.mode, AppMode::Insert | AppMode::InsertChild | AppMode::EditTodo | AppMode::Search | AppMode::EditNotes) { 3 } else { 0 }), // Input area
        ])
        .split(f.area());

    // Draw header
    draw_header(f, chunks[0], app);
    
    // Draw todos
    draw_todos(f, chunks[1], app);
    
    // Draw status bar
    draw_status_bar(f, chunks[2], app);
    
    // Draw input area if in insert, search, edit, or notes mode
    if matches!(app.mode, AppMode::Insert | AppMode::InsertChild | AppMode::EditTodo | AppMode::Search | AppMode::EditNotes) {
        draw_input(f, chunks[3], app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let colors = &app.colors;
    
    let view_name = match &app.view_mode {
        ViewMode::All => "All Todos".to_string(),
        ViewMode::Pending => "Pending Todos".to_string(),
        ViewMode::Completed => "Completed Todos".to_string(),
        ViewMode::Search(query) => format!("Search: {}", query),
        ViewMode::FilterByTag(tag) => format!("Tag: #{}", tag),
        ViewMode::FilterByContext(context) => format!("Context: @{}", context),
        ViewMode::FilterByDueDate(filter) => match filter {
            crate::todo::DueDateFilter::Overdue => "Overdue".to_string(),
            crate::todo::DueDateFilter::Today => "Due Today".to_string(),
            crate::todo::DueDateFilter::Tomorrow => "Due Tomorrow".to_string(),
            crate::todo::DueDateFilter::ThisWeek => "Due This Week".to_string(),
            crate::todo::DueDateFilter::NoDueDate => "No Due Date".to_string(),
        },
    };
    
    let mode_indicator = match app.mode {
        AppMode::Normal => ("NORMAL", colors.blue),
        AppMode::Insert => ("INSERT", colors.green),
        AppMode::InsertChild => ("ADD CHILD", colors.orange),
        AppMode::EditTodo => ("EDIT TODO", colors.yellow),
        AppMode::Search => ("SEARCH", colors.cyan),
        AppMode::TagSelection => ("TAG SELECT", colors.cyan),
        AppMode::ContextSelection => ("CONTEXT SELECT", colors.orange),
        AppMode::EditNotes => ("EDIT NOTES", colors.purple),
        AppMode::ViewNotes => ("VIEW NOTES", colors.purple),
        AppMode::TemplateSelection => ("TEMPLATE", colors.magenta),
        AppMode::RecurrenceSelection => ("RECURRENCE", colors.yellow),
        AppMode::TimeTracking => ("TIMER", colors.green),
        AppMode::WorkspaceSelection => ("WORKSPACE", colors.magenta),
        AppMode::CreateWorkspace => ("NEW WORKSPACE", colors.green),
        AppMode::Visual => ("VISUAL", colors.purple),
        AppMode::BulkOperation => ("BULK OP", colors.red),
    };
    
    // Get current workspace name
    let workspace_name = app.get_current_workspace_name();
    
    let title = Paragraph::new(format!(" Paperclip - {} | {} ", workspace_name, view_name))
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.blue))
        );
    
    f.render_widget(title, area);
    
    // Mode indicator in top right
    let mode_area = Rect {
        x: area.x + area.width.saturating_sub(12),
        y: area.y,
        width: 12,
        height: 1,
    };
    
    let mode_widget = Paragraph::new(format!(" {} ", mode_indicator.0))
        .style(Style::default().fg(colors.bg_dark).bg(mode_indicator.1).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    
    f.render_widget(mode_widget, mode_area);
}

fn draw_todos(f: &mut Frame, area: Rect, app: &mut App) {
    let colors = &app.colors;
    let todos = app.get_visible_todos();
    
    if todos.is_empty() {
        let empty_message = match &app.view_mode {
            ViewMode::All => "No todos yet. Press 'i' to add one!",
            ViewMode::Pending => "No pending todos!",
            ViewMode::Completed => "No completed todos yet.",
            ViewMode::Search(_) => "No todos found for this search.",
            ViewMode::FilterByTag(_) => "No todos found with this tag.",
            ViewMode::FilterByContext(_) => "No todos found with this context.",
            ViewMode::FilterByDueDate(_) => "No todos found for this date filter.",
        };
        
        let paragraph = Paragraph::new(empty_message)
            .style(Style::default().fg(colors.comment))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(colors.dark3))
            );
        
        f.render_widget(paragraph, area);
        return;
    }
    
    let items: Vec<ListItem> = todos
        .iter()
        .enumerate()
        .map(|(i, (todo, depth))| {
            let is_selected = i == app.selected;
            
            // Create indentation based on depth
            let indent = "  ".repeat(*depth as usize);
            
            // Tree indicators
            let tree_indicator = if *depth > 0 {
                if let Some(todo_list) = app.get_current_todo_list() {
                    if todo_list.has_children(todo.id) {
                        if todo.expanded { "└▼ " } else { "└▶ " }
                    } else {
                        "└─ "
                    }
                } else {
                    "└─ "
                }
            } else if let Some(todo_list) = app.get_current_todo_list() {
                if todo_list.has_children(todo.id) {
                    if todo.expanded { "▼ " } else { "▶ " }
                } else {
                    ""
                }
            } else {
                ""
            };
            
            // Status indicator
            let status_char = match todo.status {
                TodoStatus::Pending => if todo.is_overdue() { "!" } else { "○" },
                TodoStatus::InProgress => "◐",
                TodoStatus::Completed => "●",
            };
            
            let status_color = match todo.status {
                TodoStatus::Pending => if todo.is_overdue() { colors.red } else { colors.yellow },
                TodoStatus::InProgress => colors.blue,
                TodoStatus::Completed => colors.green,
            };
            
            // Priority indicator
            let priority_indicator = if todo.priority > 0 {
                format!(" [{}]", "!".repeat(todo.priority as usize))
            } else {
                "".to_string()
            };
            
            let priority_color = match todo.priority {
                0 => colors.fg_dark,
                1 => colors.green,
                2 => colors.yellow,
                3 => colors.orange,
                4..=5 => colors.red,
                _ => colors.fg_dark,
            };
            
            // Description style
            let desc_style = if todo.is_completed() {
                Style::default().fg(colors.comment).add_modifier(Modifier::CROSSED_OUT)
            } else if is_selected {
                Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.fg)
            };
            
            // Tags and contexts indicators
            let mut tags_contexts = Vec::new();
            
            // Add tags
            for tag in &todo.tags {
                tags_contexts.push(Span::styled(format!(" #{}", tag), Style::default().fg(colors.cyan)));
            }
            
            // Add contexts
            for context in &todo.contexts {
                tags_contexts.push(Span::styled(format!(" @{}", context), Style::default().fg(colors.orange)));
            }
            
            // Add notes indicator
            if todo.notes.is_some() && !todo.notes.as_ref().unwrap().trim().is_empty() {
                tags_contexts.push(Span::styled(" [N]".to_string(), Style::default().fg(colors.purple)));
            }
            
            // Add due date indicator
            if let Some(due) = todo.due_date {
                let now = chrono::Local::now();
                let due_text = if due.date_naive() == now.date_naive() {
                    " [today]".to_string()
                } else if due.date_naive() == now.date_naive() + chrono::Duration::days(1) {
                    " [tomorrow]".to_string()
                } else {
                    format!(" [{}]", due.format("%m/%d"))
                };
                
                let due_color = if todo.is_overdue() {
                    colors.red
                } else if due.date_naive() == now.date_naive() {
                    colors.yellow
                } else {
                    colors.blue
                };
                
                tags_contexts.push(Span::styled(due_text, Style::default().fg(due_color)));
            }
            
            let mut line_spans = vec![
                Span::styled(indent, Style::default().fg(colors.dark3)),
                Span::styled(tree_indicator, Style::default().fg(colors.cyan)),
                Span::styled(format!("{} ", status_char), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(&todo.description, desc_style),
                Span::styled(priority_indicator, Style::default().fg(priority_color).add_modifier(Modifier::BOLD)),
            ];
            
            line_spans.extend(tags_contexts);
            let line = Line::from(line_spans);
            
            ListItem::new(line)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.blue))
                .title(format!(" {} ({}) ", 
                    match &app.view_mode {
                        ViewMode::All => "All".to_string(),
                        ViewMode::Pending => "Pending".to_string(), 
                        ViewMode::Completed => "Completed".to_string(),
                        ViewMode::Search(_) => "Search".to_string(),
                        ViewMode::FilterByTag(tag) => format!("#{}", tag),
                        ViewMode::FilterByContext(context) => format!("@{}", context),
                        ViewMode::FilterByDueDate(filter) => match filter {
                            crate::todo::DueDateFilter::Overdue => "Overdue".to_string(),
                            crate::todo::DueDateFilter::Today => "Today".to_string(),
                            crate::todo::DueDateFilter::Tomorrow => "Tomorrow".to_string(),
                            crate::todo::DueDateFilter::ThisWeek => "This Week".to_string(),
                            crate::todo::DueDateFilter::NoDueDate => "No Due".to_string(),
                        },
                    },
                    todos.len()
                ))
                .title_style(Style::default().fg(colors.cyan).add_modifier(Modifier::BOLD))
        )
        .style(Style::default().fg(colors.fg));
    
    let mut list_state = ListState::default();
    list_state.select(Some(app.selected));
    
    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let colors = &app.colors;
    
    let (pending_count, completed_count, total_count) = if let Some(todo_list) = app.get_current_todo_list() {
        (todo_list.pending_count(), todo_list.completed_count(), todo_list.total_count())
    } else {
        (0, 0, 0)
    };
    
    let status_text = if let Some(msg) = &app.message {
        msg.clone()
    } else {
        format!("Total: {} | Pending: {} | Completed: {} | Press '?' for help", 
                total_count, pending_count, completed_count)
    };
    
    let paragraph = Paragraph::new(status_text)
        .style(Style::default().fg(colors.fg_dark).bg(Color::Reset))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.dark3))
        );
    
    f.render_widget(paragraph, area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
    let colors = &app.colors;
    
    let title = match app.mode {
        AppMode::Insert => " Add Todo ".to_string(),
        AppMode::InsertChild => {
            if let Some(parent_id) = app.inserting_child_for {
                if let Some(todo_list) = app.get_current_todo_list() {
                    if let Some(parent) = todo_list.get_todo(parent_id) {
                        format!(" Add Child to: {} ", parent.description)
                    } else {
                        " Add Child Todo ".to_string()
                    }
                } else {
                    " Add Child Todo ".to_string()
                }
            } else {
                " Add Child Todo ".to_string()
            }
        }
        AppMode::EditTodo => {
            if let Some(todo_id) = app.editing_todo_id {
                if let Some(todo_list) = app.get_current_todo_list() {
                    if let Some(todo) = todo_list.get_todo(todo_id) {
                        format!(" Edit Todo: {} ", todo.description)
                    } else {
                        " Edit Todo ".to_string()
                    }
                } else {
                    " Edit Todo ".to_string()
                }
            } else {
                " Edit Todo ".to_string()
            }
        }
        AppMode::Search => " Search Todos ".to_string(),
        AppMode::EditNotes => {
            if let Some(todo_id) = app.editing_notes_for {
                if let Some(todo_list) = app.get_current_todo_list() {
                    if let Some(todo) = todo_list.get_todo(todo_id) {
                        format!(" Edit Notes for: {} ", todo.description)
                    } else {
                        " Edit Notes ".to_string()
                    }
                } else {
                    " Edit Notes ".to_string()
                }
            } else {
                " Edit Notes ".to_string()
            }
        }
        _ => " Input ".to_string(),
    };
    
    let border_color = match app.mode {
        AppMode::Insert => colors.green,
        AppMode::InsertChild => colors.orange,
        AppMode::EditTodo => colors.yellow,
        AppMode::Search => colors.cyan,
        AppMode::EditNotes => colors.purple,
        _ => colors.blue,
    };
    
    let input_text = match app.mode {
        AppMode::Search => &app.search_buffer,
        AppMode::EditTodo => &app.edit_buffer,
        AppMode::EditNotes => &app.notes_buffer,
        _ => &app.input_buffer,
    };
    
    let input = Paragraph::new(input_text.as_str())
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(input, area);
    
    // Set cursor position
    let cursor_x = match app.mode {
        AppMode::Search => app.search_buffer.len(),
        AppMode::EditTodo => app.edit_buffer.len(),
        AppMode::EditNotes => app.notes_buffer.len(),
        _ => app.input_buffer.len(),
    };
    
    f.set_cursor_position((
        area.x + cursor_x as u16 + 1,
        area.y + 1,
    ));
}

fn draw_help(f: &mut Frame, app: &App) {
    let colors = &app.colors;
    
    let help_text = vec![
        Line::from(vec![Span::styled("Paperclip - Help", Style::default().fg(colors.cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("Navigation:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  j/↓     - Move down"),
        Line::from("  k/↑     - Move up"), 
        Line::from("  g       - Go to top"),
        Line::from("  G       - Go to bottom"),
        Line::from(""),
        Line::from(vec![Span::styled("Actions:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  i       - Insert new todo"),
        Line::from("  e       - Edit selected todo"),
        Line::from("  a       - Add child todo"),
        Line::from("  Space   - Toggle todo complete"),
        Line::from("  d       - Delete selected todo"),
        Line::from("  v       - Cycle view mode (all/pending/completed)"),
        Line::from(""),
        Line::from(vec![Span::styled("Search & Filter:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  /       - Search todos (by text, tags, contexts)"),
        Line::from("  #       - Select tag filter (popup with counts)"),
        Line::from("  @       - Select context filter (popup with counts)"),
        Line::from("  !       - Cycle due date filter"),
        Line::from("  Esc     - Clear filters"),
        Line::from(""),
        Line::from(vec![Span::styled("Hierarchy:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  Enter   - Expand/collapse todo"),
        Line::from("  D       - Delete todo and all children"),
        Line::from(""),
        Line::from(vec![Span::styled("Priority:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  +/=     - Increase priority (0-5 scale)"),
        Line::from("  -       - Decrease priority"),
        Line::from(""),
        Line::from(vec![Span::styled("Advanced Features:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  n       - Edit notes for selected todo"),
        Line::from("  V       - View notes for selected todo (read-only)"),
        Line::from("  t       - Toggle timer for selected todo"),
        Line::from("  T       - Apply template to new todo"),
        Line::from("  r       - Set recurrence for selected todo"),
        Line::from(""),
        Line::from(vec![Span::styled("Visual Indicators:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  ○       - Pending | ◐ In Progress | ● Completed"),
        Line::from("  !       - Overdue | ▼▶ Expandable | [!] Priority"),
        Line::from("  #tag    - Tags (cyan) | @context (orange)"),
        Line::from("  [N]     - Has notes (purple) | [today] Due dates"),
        Line::from("  [date]  - Due dates (red=overdue, yellow=today)"),
        Line::from(""),
        Line::from(vec![Span::styled("Todo Format:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  Example: 'Fix bug #urgent @work due:today'"),
        Line::from(""),
        Line::from(vec![Span::styled("Workspaces:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  w       - Switch workspace (popup selection)"),
        Line::from("  In workspace selection popup:"),
        Line::from("    n     - Create new workspace"),
        Line::from("    d     - Delete selected workspace"),
        Line::from("    Enter - Select workspace"),
        Line::from(""),
        Line::from(vec![Span::styled("Other:", Style::default().fg(colors.blue).add_modifier(Modifier::BOLD))]),
        Line::from("  ?       - Toggle this help"),
        Line::from("  q       - Quit"),
        Line::from(""),
        Line::from(vec![Span::styled("In popups: j/k to navigate, Enter to select, Esc to cancel", Style::default().fg(colors.comment))]),
    ];
    
    let help_widget = Paragraph::new(help_text)
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.cyan))
                .title(" Help ")
                .title_style(Style::default().fg(colors.cyan).add_modifier(Modifier::BOLD))
        );
    
    // Center the help dialog
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);
    f.render_widget(help_widget, area);
}

fn draw_selection_popup(f: &mut Frame, app: &App) {
    let colors = &app.colors;
    
    let (items, title, border_color) = match app.mode {
        AppMode::TagSelection => {
            let tag_counts = if let Some(todo_list) = app.get_current_todo_list() {
                todo_list.get_tag_counts()
            } else {
                Vec::new()
            };
            let items: Vec<ListItem> = tag_counts.iter()
                .enumerate()
                .map(|(i, (tag, count))| {
                    let is_selected = i == app.popup_selected;
                    let style = if is_selected {
                        Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg)
                    };
                    
                    let line = Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled("#", Style::default().fg(colors.cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} ", tag), style),
                        Span::styled(format!("({})", count), Style::default().fg(colors.comment)),
                    ]);
                    
                    ListItem::new(line)
                })
                .collect();
            (items, " Select Tag ", colors.cyan)
        }
        AppMode::ContextSelection => {
            let context_counts = if let Some(todo_list) = app.get_current_todo_list() {
                todo_list.get_context_counts()
            } else {
                Vec::new()
            };
            let items: Vec<ListItem> = context_counts.iter()
                .enumerate()
                .map(|(i, (context, count))| {
                    let is_selected = i == app.popup_selected;
                    let style = if is_selected {
                        Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg)
                    };
                    
                    let line = Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled("@", Style::default().fg(colors.orange).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} ", context), style),
                        Span::styled(format!("({})", count), Style::default().fg(colors.comment)),
                    ]);
                    
                    ListItem::new(line)
                })
                .collect();
            (items, " Select Context ", colors.orange)
        }
        AppMode::TemplateSelection => {
            let templates = app.template_manager.get_all_templates();
            let items: Vec<ListItem> = templates.iter()
                .enumerate()
                .map(|(i, template)| {
                    let is_selected = i == app.popup_selected;
                    let style = if is_selected {
                        Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg)
                    };
                    
                    let line = Line::from(vec![
                        Span::styled("  [T] ", Style::default().fg(colors.magenta)),
                        Span::styled(&template.name, style),
                    ]);
                    
                    ListItem::new(line)
                })
                .collect();
            (items, " Select Template ", colors.magenta)
        }
        AppMode::RecurrenceSelection => {
            let items: Vec<ListItem> = app.available_recurrence.iter()
                .enumerate()
                .map(|(i, pattern)| {
                    let is_selected = i == app.popup_selected;
                    let style = if is_selected {
                        Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg)
                    };
                    
                    let pattern_name = match pattern {
                        crate::todo::RecurrencePattern::None => "None",
                        crate::todo::RecurrencePattern::Daily => "Daily",
                        crate::todo::RecurrencePattern::Weekly => "Weekly",
                        crate::todo::RecurrencePattern::Monthly => "Monthly",
                        crate::todo::RecurrencePattern::Yearly => "Yearly",
                        crate::todo::RecurrencePattern::Custom(_) => "Custom",
                    };
                    
                    let line = Line::from(vec![
                        Span::styled("  [R] ", Style::default().fg(colors.yellow)),
                        Span::styled(pattern_name, style),
                    ]);
                    
                    ListItem::new(line)
                })
                .collect();
            (items, " Select Recurrence ", colors.yellow)
        }
        AppMode::WorkspaceSelection => {
            let items: Vec<ListItem> = app.available_workspaces.iter()
                .enumerate()
                .map(|(i, workspace_name)| {
                    let is_selected = i == app.popup_selected;
                    let style = if is_selected {
                        Style::default().fg(colors.fg).bg(colors.bg_highlight).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg)
                    };
                    
                    let line = Line::from(vec![
                        Span::styled("  [W] ", Style::default().fg(colors.magenta)),
                        Span::styled(workspace_name, style),
                    ]);
                    
                    ListItem::new(line)
                })
                .collect();
            (items, " Select Workspace ", colors.magenta)
        }
        _ => return,
    };
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        )
        .style(Style::default().fg(colors.fg));
    
    // Center the popup
    let popup_area = centered_rect(40, 60, f.area());
    f.render_widget(Clear, popup_area);
    
    let mut list_state = ListState::default();
    list_state.select(Some(app.popup_selected));
    
    f.render_stateful_widget(list, popup_area, &mut list_state);
    
    // Add instructions at the bottom of popup
    let instructions_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + popup_area.height - 2,
        width: popup_area.width - 2,
        height: 1,
    };
    
    let instructions = match app.mode {
        AppMode::WorkspaceSelection => "Enter: Select | n: New | d: Delete | Esc: Cancel | j/k: Navigate",
        _ => "Enter: Select | Esc: Cancel | j/k: Navigate",
    };
    
    let instructions_widget = Paragraph::new(instructions)
        .style(Style::default().fg(colors.comment))
        .alignment(Alignment::Center);
    
    f.render_widget(instructions_widget, instructions_area);
}

fn draw_notes_editor(f: &mut Frame, app: &App) {
    let colors = &app.colors;
    
    // Get the todo being edited
    let todo = if let Some(todo_id) = app.editing_notes_for {
        if let Some(todo_list) = app.get_current_todo_list() {
            todo_list.get_todo(todo_id)
        } else {
            return;
        }
    } else {
        return;
    };
    
    let todo = match todo {
        Some(t) => t,
        None => return,
    };
    
    // Create a larger popup for notes editing
    let popup_area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, popup_area);
    
    // Split the popup into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Todo info header
            Constraint::Min(0),    // Notes text area
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);
    
    // Draw todo info header
    let todo_info = Paragraph::new(format!("Todo: {}", todo.description))
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.purple))
                .title(" Edit Notes ")
                .title_style(Style::default().fg(colors.purple).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(todo_info, chunks[0]);
    
    // Draw notes text area
    let notes_text = if app.notes_buffer.is_empty() {
        "Type your notes here...".to_string()
    } else {
        app.notes_buffer.clone()
    };
    
    let notes_editor = Paragraph::new(notes_text)
        .style(Style::default().fg(if app.notes_buffer.is_empty() { colors.comment } else { colors.fg }).bg(Color::Reset))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(colors.purple))
        );
    
    f.render_widget(notes_editor, chunks[1]);
    
    // Draw instructions
    let instructions = Paragraph::new("F2, Ctrl+Enter, or Ctrl+S: Save | Esc: Cancel | Enter: New line")
        .style(Style::default().fg(colors.comment))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.purple))
        );
    
    f.render_widget(instructions, chunks[2]);
    
    // Set cursor position in the notes area
    // Calculate cursor position based on text content and wrapping
    let text_area = Rect {
        x: chunks[1].x + 1,
        y: chunks[1].y,
        width: chunks[1].width - 2,
        height: chunks[1].height,
    };
    
    // For simplicity, put cursor at end of last line
    let lines: Vec<&str> = app.notes_buffer.split('\n').collect();
    let cursor_y = text_area.y + lines.len().saturating_sub(1) as u16;
    let cursor_x = if let Some(last_line) = lines.last() {
        text_area.x + last_line.len() as u16
    } else {
        text_area.x
    };
    
    // Make sure cursor stays within bounds
    let cursor_x = cursor_x.min(text_area.x + text_area.width - 1);
    let cursor_y = cursor_y.min(text_area.y + text_area.height - 1);
    
    f.set_cursor_position((cursor_x, cursor_y));
}

fn draw_notes_viewer(f: &mut Frame, app: &App) {
    let colors = &app.colors;
    
    // Get the todo being viewed
    let todo = if let Some(todo_id) = app.editing_notes_for {
        if let Some(todo_list) = app.get_current_todo_list() {
            todo_list.get_todo(todo_id)
        } else {
            return;
        }
    } else {
        return;
    };
    
    let todo = match todo {
        Some(t) => t,
        None => return,
    };
    
    // Create a larger popup for notes viewing
    let popup_area = centered_rect(70, 70, f.area());
    f.render_widget(Clear, popup_area);
    
    // Split the popup into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Todo info header
            Constraint::Min(0),    // Notes text area
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);
    
    // Draw todo info header
    let todo_info = Paragraph::new(format!("Todo: {}", todo.description))
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.purple))
                .title(" View Notes ")
                .title_style(Style::default().fg(colors.purple).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(todo_info, chunks[0]);
    
    // Draw notes text area (read-only)
    let notes_text = app.notes_buffer.clone();
    
    let notes_viewer = Paragraph::new(notes_text)
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(colors.purple))
        );
    
    f.render_widget(notes_viewer, chunks[1]);
    
    // Draw instructions
    let instructions = Paragraph::new("Esc: Close | n: Edit notes")
        .style(Style::default().fg(colors.comment))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.purple))
        );
    
    f.render_widget(instructions, chunks[2]);
}

fn draw_create_workspace_ui(f: &mut Frame, app: &App) {
    let colors = &app.colors;
    
    // Create centered popup for workspace creation
    let popup_area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, popup_area);
    
    // Split the popup into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Input field
            Constraint::Length(3), // Instructions
        ])
        .split(popup_area);
    
    // Draw header
    let header = Paragraph::new("Create New Workspace")
        .style(Style::default().fg(colors.fg).bg(Color::Reset))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.green))
                .title(" New Workspace ")
                .title_style(Style::default().fg(colors.green).add_modifier(Modifier::BOLD))
        );
    
    f.render_widget(header, chunks[0]);
    
    // Draw input field
    let input_text = if app.input_buffer.is_empty() {
        "Enter workspace name...".to_string()
    } else {
        app.input_buffer.clone()
    };
    
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(if app.input_buffer.is_empty() { colors.comment } else { colors.fg }).bg(Color::Reset))
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(colors.green))
        );
    
    f.render_widget(input, chunks[1]);
    
    // Draw instructions
    let instructions = Paragraph::new("Enter: Create | Esc: Cancel")
        .style(Style::default().fg(colors.comment))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.green))
        );
    
    f.render_widget(instructions, chunks[2]);
    
    // Set cursor position in the input field
    let cursor_x = chunks[1].x + 1 + app.input_buffer.len() as u16;
    let cursor_y = chunks[1].y + 1;
    
    f.set_cursor_position((cursor_x, cursor_y));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
