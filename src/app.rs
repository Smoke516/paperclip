use crate::colors::TokyoNightColors;
use crate::todo::{Todo, TodoList, DueDateFilter, RecurrencePattern, WorkspaceManager};
use crate::template::TemplateManager;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    AddTodo { workspace_id: String, todo: Todo },
    DeleteTodo { workspace_id: String, todo: Todo },
    CompleteTodo { workspace_id: String, todo_id: u32, old_status: crate::todo::TodoStatus },
    EditTodo { workspace_id: String, todo_id: u32, old_description: String, old_raw_description: String },
    ChangePriority { workspace_id: String, todo_id: u32, old_priority: u8 },
    AddChildTodo { workspace_id: String, parent_id: u32, child_todo: Todo },
    DeleteWithChildren { workspace_id: String, deleted_todos: Vec<Todo> },
}

pub struct CommandHistory {
    undo_stack: VecDeque<Command>,
    redo_stack: VecDeque<Command>,
    max_history: usize,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history: 50, // Store last 50 commands
        }
    }
    
    pub fn push_command(&mut self, command: Command) {
        // Clear redo stack when new command is executed
        self.redo_stack.clear();
        
        self.undo_stack.push_back(command);
        
        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }
    }
    
    pub fn undo(&mut self) -> Option<Command> {
        if let Some(command) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(command.clone());
            Some(command)
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<Command> {
        if let Some(command) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(command.clone());
            Some(command)
        } else {
            None
        }
    }
    
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Welcome,
    Normal,
    Insert,
    InsertChild,
    EditTodo,
    Search,
    TagSelection,
    ContextSelection,
    // Advanced feature modes
    EditNotes,
    ViewNotes,
    TemplateSelection,
    RecurrenceSelection,
    TimeTracking,
    WorkspaceSelection,
    CreateWorkspace,
    // Bulk operations
    Visual,
    BulkOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    All,
    Pending,
    Completed,
    Search(String),
    FilterByTag(String),
    FilterByContext(String),
    FilterByDueDate(DueDateFilter),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BulkOperationType {
    Complete,
    Delete,
    SetPriority(u8),
    AddTag(String),
    AddContext(String),
    MoveTo(String), // Move to different workspace
}

pub struct App {
    pub workspace_manager: WorkspaceManager,
    pub mode: AppMode,
    pub view_mode: ViewMode,
    pub selected: usize,
    pub input_buffer: String,
    pub search_buffer: String,
    pub search_cursor_pos: usize, // Cursor position in search buffer
    pub colors: TokyoNightColors,
    pub should_quit: bool,
    pub show_help: bool,
    pub message: Option<String>,
    pub inserting_child_for: Option<u32>, // Track which todo we're adding a child for
    // Selection popup state
    pub popup_selected: usize,
    pub available_tags: Vec<String>,
    pub available_contexts: Vec<String>,
    
    // Advanced features
    pub template_manager: TemplateManager,
    pub notes_buffer: String, // For editing notes
    pub notes_cursor_pos: usize, // Cursor position in notes buffer
    pub editing_notes_for: Option<u32>, // Which todo's notes we're editing
    pub edit_buffer: String, // For editing todo descriptions
    pub edit_cursor_pos: usize, // Cursor position in edit buffer
    pub editing_todo_id: Option<u32>, // Which todo's description we're editing
    pub input_cursor_pos: usize, // Cursor position in input buffer
    pub available_templates: Vec<String>, // Template IDs for selection
    pub available_recurrence: Vec<RecurrencePattern>, // For recurrence selection
    
    // Workspace management
    pub available_workspaces: Vec<String>, // Workspace IDs for selection
    
    // Command history for undo/redo
    pub command_history: CommandHistory,
    
    // Bulk operations
    pub selected_todos: std::collections::HashSet<u32>,
    pub visual_start: Option<usize>, // Starting position for visual selection
    pub bulk_operation: Option<BulkOperationType>,
    
    // Welcome screen
    pub welcome_selected: usize, // Selected option on welcome screen
    pub is_first_launch: bool, // Track if this is the first time using the app
}

impl App {
    pub fn new() -> Self {
        let mut workspace_manager = WorkspaceManager::new();
        // Create initial workspace
        workspace_manager.create_workspace("Personal".to_string(), Some("Your personal todos".to_string()));
        
        // Get available workspace names for initial selection
        let available_workspaces: Vec<String> = workspace_manager.get_all_workspaces()
            .iter()
            .map(|ws| ws.name.clone())
            .collect();
        
        // Check if this is first launch (no workspaces beyond default means first time)
        let is_first_launch = available_workspaces.len() <= 1;
        
        Self {
            workspace_manager,
            mode: if is_first_launch { AppMode::Welcome } else { AppMode::WorkspaceSelection },
            view_mode: ViewMode::All,
            selected: 0,
            input_buffer: String::new(),
            search_buffer: String::new(),
            search_cursor_pos: 0,
            colors: TokyoNightColors::new(),
            should_quit: false,
            show_help: false,
            message: Some("Select a workspace to get started".to_string()),
            inserting_child_for: None,
            popup_selected: 0,
            available_tags: Vec::new(),
            available_contexts: Vec::new(),
            
            // Initialize advanced features
            template_manager: TemplateManager::with_builtin_templates(),
            notes_buffer: String::new(),
            notes_cursor_pos: 0,
            editing_notes_for: None,
            edit_buffer: String::new(),
            edit_cursor_pos: 0,
            editing_todo_id: None,
            input_cursor_pos: 0,
            available_templates: Vec::new(),
            available_recurrence: vec![
                RecurrencePattern::None,
                RecurrencePattern::Daily,
                RecurrencePattern::Weekly,
                RecurrencePattern::Monthly,
                RecurrencePattern::Yearly,
            ],
            available_workspaces,
            command_history: CommandHistory::new(),
            selected_todos: std::collections::HashSet::new(),
            visual_start: None,
            bulk_operation: None,
            welcome_selected: 0,
            is_first_launch,
        }
    }
    
    // Bulk operations functionality
    pub fn enter_visual_mode(&mut self) {
        self.mode = AppMode::Visual;
        self.visual_start = Some(self.selected);
        self.selected_todos.clear();
        // Add current selection to bulk selection
        if let Some(id) = self.get_selected_todo_id() {
            self.selected_todos.insert(id);
        }
        self.set_message("Visual mode - use j/k to select, Space to toggle, Enter to apply operation".to_string());
    }
    
    pub fn exit_visual_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.visual_start = None;
        self.selected_todos.clear();
        self.bulk_operation = None;
    }
    
    pub fn toggle_selection_in_visual(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if self.selected_todos.contains(&id) {
                self.selected_todos.remove(&id);
            } else {
                self.selected_todos.insert(id);
            }
        }
    }
    
    pub fn select_range_in_visual(&mut self) {
        if let Some(start) = self.visual_start {
            let end = self.selected;
            let todos = self.get_visible_todos();
            
            let (start_idx, end_idx) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            
            // Collect todo IDs first to avoid borrowing issues
            let mut todo_ids = Vec::new();
            for i in start_idx..=end_idx.min(todos.len().saturating_sub(1)) {
                if let Some((todo, _)) = todos.get(i) {
                    todo_ids.push(todo.id);
                }
            }
            
            // Clear current selection and add range
            self.selected_todos.clear();
            for id in todo_ids {
                self.selected_todos.insert(id);
            }
        }
    }
    
    pub fn bulk_complete_todos(&mut self) {
        if self.selected_todos.is_empty() {
            self.set_message("No todos selected for bulk operation".to_string());
            return;
        }
        
        let mut completed_count = 0;
        let selected_ids: Vec<u32> = self.selected_todos.iter().cloned().collect();
        
        if let Some(todo_list) = self.get_current_todo_list_mut() {
            for id in selected_ids {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    if !todo.is_completed() {
                        todo.complete();
                        completed_count += 1;
                    }
                }
            }
        }
        
        self.set_message(format!("Bulk completed {} todos", completed_count));
        self.exit_visual_mode();
    }
    
    pub fn bulk_delete_todos(&mut self) {
        if self.selected_todos.is_empty() {
            self.set_message("No todos selected for bulk operation".to_string());
            return;
        }
        
        let mut deleted_todos = Vec::new();
        let selected_ids: Vec<u32> = self.selected_todos.iter().cloned().collect();
        
        if let Some(todo_list) = self.get_current_todo_list_mut() {
            for id in selected_ids {
                if let Some(todo) = todo_list.get_todo(id).cloned() {
                    deleted_todos.push(todo);
                    todo_list.remove_todo(id);
                }
            }
        }
        
        // Record command for undo
        if !deleted_todos.is_empty() {
            if let Some(workspace_id) = self.workspace_manager.get_current_workspace_id() {
                let command = Command::DeleteWithChildren { workspace_id, deleted_todos: deleted_todos.clone() };
                self.command_history.push_command(command);
            }
        }
        
        let count = deleted_todos.len();
        self.set_message(format!("Bulk deleted {} todos. Press 'u' to undo.", count));
        self.exit_visual_mode();
        
        // Adjust selection after deletion
        let todos = self.get_visible_todos();
        if self.selected >= todos.len() && todos.len() > 0 {
            self.selected = todos.len() - 1;
        }
    }
    
    pub fn bulk_set_priority(&mut self, priority: u8) {
        if self.selected_todos.is_empty() {
            self.set_message("No todos selected for bulk operation".to_string());
            return;
        }
        
        let mut updated_count = 0;
        let selected_ids: Vec<u32> = self.selected_todos.iter().cloned().collect();
        
        if let Some(todo_list) = self.get_current_todo_list_mut() {
            for id in selected_ids {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    todo.priority = priority;
                    updated_count += 1;
                }
            }
        }
        
        self.set_message(format!("Set priority to {} for {} todos", priority, updated_count));
        self.exit_visual_mode();
    }
    
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn set_message(&mut self, msg: String) {
        self.message = Some(msg);
    }

    // Workspace helper methods
    pub fn get_current_todo_list(&self) -> Option<&TodoList> {
        self.workspace_manager.get_current_todo_list()
    }
    
    pub fn get_current_todo_list_mut(&mut self) -> Option<&mut TodoList> {
        self.workspace_manager.get_current_todo_list_mut()
    }
    
    pub fn get_current_workspace_name(&self) -> String {
        self.workspace_manager.get_current_workspace()
            .map(|ws| ws.name.clone())
            .unwrap_or_else(|| "No Workspace".to_string())
    }

    pub fn get_visible_todos(&self) -> Vec<(&Todo, u32)> {
        let todo_list = match self.get_current_todo_list() {
            Some(list) => list,
            None => return Vec::new(),
        };
        
        match &self.view_mode {
            ViewMode::All => todo_list.get_flattened_todos(),
            ViewMode::Pending => todo_list.get_flattened_pending_todos(),
            ViewMode::Completed => todo_list.get_flattened_completed_todos(),
            ViewMode::Search(query) => {
                // Search across all workspaces as requested
                let search_results = self.workspace_manager.search_all_workspaces(query);
                let mut all_results = Vec::new();
                for (_, results) in search_results {
                    all_results.extend(results);
                }
                all_results
            },
            ViewMode::FilterByTag(tag) => todo_list.filter_by_tag(tag),
            ViewMode::FilterByContext(context) => todo_list.filter_by_context(context),
            ViewMode::FilterByDueDate(filter) => todo_list.filter_by_due_date(*filter),
        }
    }

    pub fn get_selected_todo_id(&self) -> Option<u32> {
        let todos = self.get_visible_todos();
        todos.get(self.selected).map(|(todo, _)| todo.id)
    }

    pub fn move_selection_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let todos = self.get_visible_todos();
        if self.selected < todos.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn go_to_top(&mut self) {
        self.selected = 0;
    }

    pub fn go_to_bottom(&mut self) {
        let todos = self.get_visible_todos();
        self.selected = todos.len().saturating_sub(1);
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = AppMode::Insert;
        self.clear_input_buffer();
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.clear_input_buffer();
        self.inserting_child_for = None;
    }

    pub fn submit_input(&mut self) {
        if !self.input_buffer.trim().is_empty() {
            let input_text = self.input_buffer.trim().to_string();
            match self.mode {
                AppMode::Insert => {
                    // Get workspace ID before borrowing todo_list mutably
                    let workspace_id = self.workspace_manager.get_current_workspace_id();
                    
                    if let Some(todo_list) = self.get_current_todo_list_mut() {
                        let todo_id = todo_list.add_todo(input_text.clone());
                        
                        // Clone the todo for undo command after it's created
                        let todo_for_undo = todo_list.get_todo(todo_id).cloned();
                        
                        self.set_message("Todo added! Press 'u' to undo.".to_string());
                        
                        // Record command for undo after releasing the mutable borrow
                        if let (Some(todo), Some(ws_id)) = (todo_for_undo, workspace_id) {
                            let command = Command::AddTodo { workspace_id: ws_id, todo };
                            self.command_history.push_command(command);
                        }
                    } else {
                        self.set_message("No workspace selected".to_string());
                    }
                }
                AppMode::InsertChild => {
                    if let Some(parent_id) = self.inserting_child_for {
                        if let Some(todo_list) = self.get_current_todo_list_mut() {
                            if let Some(_) = todo_list.add_child_todo(parent_id, input_text) {
                                self.set_message("Child todo added!".to_string());
                            } else {
                                self.set_message("Failed to add child todo".to_string());
                            }
                        } else {
                            self.set_message("No workspace selected".to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        self.enter_normal_mode();
    }

    pub fn toggle_todo_complete(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            let workspace_id = self.workspace_manager.get_current_workspace_id();
            
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    // Record the old status for undo
                    let old_status = todo.status.clone();
                    
                    todo.toggle_complete();
                    let status = if todo.is_completed() { "completed" } else { "pending" };
                    
                    // Record command for undo
                    if let Some(ws_id) = workspace_id {
                        let command = Command::CompleteTodo { workspace_id: ws_id, todo_id: id, old_status };
                        self.command_history.push_command(command);
                    }
                    
                    self.set_message(format!("Todo marked as {}. Press 'u' to undo.", status));
                }
            }
        }
    }

    pub fn delete_selected_todo(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                todo_list.remove_todo(id);
                self.set_message("Todo deleted!".to_string());
                
                // Adjust selection if needed
                let todos = self.get_visible_todos();
                if self.selected >= todos.len() && todos.len() > 0 {
                    self.selected = todos.len() - 1;
                }
            }
        }
    }

    pub fn cycle_view_mode(&mut self) {
        self.view_mode = match &self.view_mode {
            ViewMode::All => ViewMode::Pending,
            ViewMode::Pending => ViewMode::Completed,
            ViewMode::Completed => ViewMode::All,
            _ => ViewMode::All, // Reset to All from any filter mode
        };
        self.selected = 0; // Reset selection when changing view
        
        let view_name = self.get_view_name();
        self.set_message(format!("Viewing {}", view_name));
    }
    
    pub fn get_view_name(&self) -> &str {
        match &self.view_mode {
            ViewMode::All => "all todos",
            ViewMode::Pending => "pending todos", 
            ViewMode::Completed => "completed todos",
            ViewMode::Search(_query) => "search results",
            ViewMode::FilterByTag(_tag) => "filtered by tag",
            ViewMode::FilterByContext(_context) => "filtered by context",
            ViewMode::FilterByDueDate(filter) => match filter {
                DueDateFilter::Overdue => "overdue todos",
                DueDateFilter::Today => "due today",
                DueDateFilter::Tomorrow => "due tomorrow",
                DueDateFilter::ThisWeek => "due this week",
                DueDateFilter::NoDueDate => "no due date",
            },
        }
    }

    pub fn increase_priority(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    if todo.priority < 5 {
                        todo.priority += 1;
                        let priority = todo.priority;
                        self.set_message(format!("Priority increased to {}", priority));
                    }
                }
            }
        }
    }

    pub fn decrease_priority(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    if todo.priority > 0 {
                        todo.priority -= 1;
                        let priority = todo.priority;
                        self.set_message(format!("Priority decreased to {}", priority));
                    }
                }
            }
        }
    }

    // Hierarchical methods
    pub fn add_child_todo(&mut self) {
        if let Some(parent_id) = self.get_selected_todo_id() {
            self.mode = AppMode::InsertChild;
            self.clear_input_buffer();
            self.inserting_child_for = Some(parent_id);
        }
    }

    pub fn toggle_expansion(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if todo_list.has_children(id) {
                    todo_list.toggle_expanded(id);
                    let expanded = todo_list.get_todo(id).map(|t| t.expanded).unwrap_or(false);
                    let action = if expanded { "expanded" } else { "collapsed" };
                    self.set_message(format!("Todo {}", action));
                }
            }
        }
    }

    pub fn delete_selected_with_children(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                let removed = todo_list.remove_todo_and_children(id);
                let count = removed.len();
                if count == 1 {
                    self.set_message("Todo deleted!".to_string());
                } else {
                    self.set_message(format!("Todo and {} children deleted!", count - 1));
                }
                
                // Adjust selection if needed
                let todos = self.get_visible_todos();
                if self.selected >= todos.len() && todos.len() > 0 {
                    self.selected = todos.len() - 1;
                }
            }
        }
    }

    // Search and filter methods
    pub fn enter_search_mode(&mut self) {
        self.mode = AppMode::Search;
        self.search_buffer.clear();
        self.search_cursor_pos = 0;
    }

    pub fn submit_search(&mut self) {
        if self.search_buffer.trim().is_empty() {
            self.view_mode = ViewMode::All;
        } else {
            self.view_mode = ViewMode::Search(self.search_buffer.trim().to_string());
        }
        self.selected = 0;
        self.mode = AppMode::Normal;
        self.set_message(format!("Searching for: {}", self.search_buffer));
        self.search_buffer.clear();
        self.search_cursor_pos = 0;
    }

    pub fn add_char_to_search(&mut self, c: char) {
        self.search_buffer.insert(self.search_cursor_pos, c);
        self.search_cursor_pos += c.len_utf8();
    }

    pub fn remove_char_from_search(&mut self) {
        if self.search_cursor_pos > 0 {
            // Find the start of the character to remove (handle UTF-8)
            let mut char_start = self.search_cursor_pos - 1;
            while char_start > 0 && !self.search_buffer.is_char_boundary(char_start) {
                char_start -= 1;
            }
            
            self.search_buffer.remove(char_start);
            self.search_cursor_pos = char_start;
        }
    }
    
    // Search cursor navigation
    pub fn move_search_cursor_left(&mut self) {
        if self.search_cursor_pos > 0 {
            self.search_cursor_pos -= 1;
            // Ensure we're at a valid character boundary
            while self.search_cursor_pos > 0 && !self.search_buffer.is_char_boundary(self.search_cursor_pos) {
                self.search_cursor_pos -= 1;
            }
        }
    }
    
    pub fn move_search_cursor_right(&mut self) {
        if self.search_cursor_pos < self.search_buffer.len() {
            self.search_cursor_pos += 1;
            // Ensure we're at a valid character boundary
            while self.search_cursor_pos < self.search_buffer.len() && !self.search_buffer.is_char_boundary(self.search_cursor_pos) {
                self.search_cursor_pos += 1;
            }
        }
    }

    pub fn clear_filters(&mut self) {
        self.view_mode = ViewMode::All;
        self.selected = 0;
        self.set_message("Filters cleared".to_string());
    }

    pub fn enter_tag_selection(&mut self) {
        if let Some(todo_list) = self.get_current_todo_list() {
            self.available_tags = todo_list.get_all_tags();
            if self.available_tags.is_empty() {
                self.set_message("No tags found".to_string());
                return;
            }
            self.mode = AppMode::TagSelection;
            self.popup_selected = 0;
        } else {
            self.set_message("No workspace selected".to_string());
        }
    }

    pub fn enter_context_selection(&mut self) {
        if let Some(todo_list) = self.get_current_todo_list() {
            self.available_contexts = todo_list.get_all_contexts();
            if self.available_contexts.is_empty() {
                self.set_message("No contexts found".to_string());
                return;
            }
            self.mode = AppMode::ContextSelection;
            self.popup_selected = 0;
        } else {
            self.set_message("No workspace selected".to_string());
        }
    }

    pub fn move_popup_selection_up(&mut self) {
        if self.popup_selected > 0 {
            self.popup_selected -= 1;
        }
    }

    pub fn move_popup_selection_down(&mut self) {
        let max_items = match self.mode {
            AppMode::TagSelection => self.available_tags.len(),
            AppMode::ContextSelection => self.available_contexts.len(),
            AppMode::TemplateSelection => self.available_templates.len(),
            AppMode::RecurrenceSelection => self.available_recurrence.len(),
            AppMode::WorkspaceSelection => self.available_workspaces.len() + 1, // +1 for Home option
            _ => 0,
        };
        if self.popup_selected < max_items.saturating_sub(1) {
            self.popup_selected += 1;
        }
    }

    pub fn select_from_popup(&mut self) {
        match self.mode {
            AppMode::TagSelection => {
                if let Some(tag) = self.available_tags.get(self.popup_selected) {
                    self.view_mode = ViewMode::FilterByTag(tag.clone());
                    self.selected = 0;
                    self.set_message(format!("Filtering by tag: #{}", tag));
                }
            }
            AppMode::ContextSelection => {
                if let Some(context) = self.available_contexts.get(self.popup_selected) {
                    self.view_mode = ViewMode::FilterByContext(context.clone());
                    self.selected = 0;
                    self.set_message(format!("Filtering by context: @{}", context));
                }
            }
            AppMode::TemplateSelection => {
                self.apply_template();
                return;
            }
            AppMode::RecurrenceSelection => {
                self.apply_recurrence();
                return;
            }
            AppMode::WorkspaceSelection => {
                self.switch_workspace();
                return;
            }
            _ => {}
        }
        self.mode = AppMode::Normal;
    }

    pub fn cancel_popup(&mut self) {
        self.mode = AppMode::Normal;
        self.popup_selected = 0;
        self.available_tags.clear();
        self.available_contexts.clear();
        self.available_templates.clear();
        self.available_workspaces.clear();
        self.exit_notes_mode(); // Also handles notes mode cancellation
    }

    pub fn cycle_due_date_filter(&mut self) {
        use crate::todo::DueDateFilter;
        let next_filter = match &self.view_mode {
            ViewMode::FilterByDueDate(DueDateFilter::Overdue) => DueDateFilter::Today,
            ViewMode::FilterByDueDate(DueDateFilter::Today) => DueDateFilter::Tomorrow,
            ViewMode::FilterByDueDate(DueDateFilter::Tomorrow) => DueDateFilter::ThisWeek,
            ViewMode::FilterByDueDate(DueDateFilter::ThisWeek) => DueDateFilter::NoDueDate,
            ViewMode::FilterByDueDate(DueDateFilter::NoDueDate) => DueDateFilter::Overdue,
            _ => DueDateFilter::Overdue,
        };
        
        self.view_mode = ViewMode::FilterByDueDate(next_filter);
        self.selected = 0;
        let filter_name = self.get_view_name();
        self.set_message(format!("Filtering by: {}", filter_name));
    }
    
    // Advanced feature methods
    
    // Time tracking
    pub fn toggle_timer(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo(id) {
                    if todo.is_timer_running() {
                        todo_list.stop_timer(id);
                        self.set_message("Timer stopped".to_string());
                    } else {
                        todo_list.start_timer(id);
                        self.set_message("Timer started".to_string());
                    }
                }
            }
        }
    }
    
    // Notes editing
    pub fn enter_notes_mode(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            self.editing_notes_for = Some(id);
            self.mode = AppMode::EditNotes;
            
            // Load existing notes into buffer
            if let Some(todo_list) = self.get_current_todo_list() {
                if let Some(todo) = todo_list.get_todo(id) {
                    self.notes_buffer = todo.notes.clone().unwrap_or_default();
                    self.notes_cursor_pos = self.notes_buffer.len();
                } else {
                    self.notes_buffer.clear();
                    self.notes_cursor_pos = 0;
                }
            } else {
                self.notes_buffer.clear();
                self.notes_cursor_pos = 0;
            }
        }
    }
    
    pub fn save_notes(&mut self) {
        if let Some(id) = self.editing_notes_for {
            let notes = if self.notes_buffer.trim().is_empty() {
                None
            } else {
                Some(self.notes_buffer.trim().to_string())
            };
            
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    todo.set_notes(notes);
                    self.set_message("Notes saved".to_string());
                }
            }
        }
        self.exit_notes_mode();
    }
    
    pub fn exit_notes_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.notes_buffer.clear();
        self.notes_cursor_pos = 0;
        self.editing_notes_for = None;
    }
    
    // Notes viewing (read-only)
    pub fn enter_view_notes_mode(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(todo_list) = self.get_current_todo_list() {
                if let Some(todo) = todo_list.get_todo(id) {
                    if todo.notes.is_some() && !todo.notes.as_ref().unwrap().trim().is_empty() {
                        let notes = todo.notes.clone().unwrap_or_default();
                        self.editing_notes_for = Some(id);
                        self.mode = AppMode::ViewNotes;
                        // Load notes into buffer for display purposes only
                        self.notes_buffer = notes;
                    } else {
                        self.set_message("This todo has no notes".to_string());
                    }
                }
            }
        }
    }
    
    pub fn exit_view_notes_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.notes_buffer.clear();
        self.editing_notes_for = None;
    }
    
    pub fn add_char_to_notes(&mut self, c: char) {
        if c == '\n' || c.is_control() {
            // Handle newlines and control characters at cursor position
            self.notes_buffer.insert(self.notes_cursor_pos, c);
            self.notes_cursor_pos += c.len_utf8();
        } else {
            // Insert regular characters at cursor position
            self.notes_buffer.insert(self.notes_cursor_pos, c);
            self.notes_cursor_pos += c.len_utf8();
        }
    }
    
    pub fn remove_char_from_notes(&mut self) {
        if self.notes_cursor_pos > 0 {
            // Find the start of the character to remove (handle UTF-8)
            let mut char_start = self.notes_cursor_pos - 1;
            while char_start > 0 && !self.notes_buffer.is_char_boundary(char_start) {
                char_start -= 1;
            }
            
            self.notes_buffer.remove(char_start);
            self.notes_cursor_pos = char_start;
        }
    }
    
    // Todo description editing
    pub fn enter_edit_mode(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            self.editing_todo_id = Some(id);
            self.mode = AppMode::EditTodo;
            
            // Load existing raw description into edit buffer
            if let Some(todo_list) = self.get_current_todo_list() {
                if let Some(todo) = todo_list.get_todo(id) {
                    self.edit_buffer = todo.raw_description.clone();
                    self.edit_cursor_pos = self.edit_buffer.len();
                } else {
                    self.edit_buffer.clear();
                    self.edit_cursor_pos = 0;
                }
            } else {
                self.edit_buffer.clear();
                self.edit_cursor_pos = 0;
            }
        }
    }
    
    pub fn save_todo_edit(&mut self) {
        if let Some(id) = self.editing_todo_id {
            if !self.edit_buffer.trim().is_empty() {
                let new_description = self.edit_buffer.trim().to_string();
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(id) {
                        todo.update_description(new_description);
                        self.set_message("Todo updated".to_string());
                    } else {
                        self.set_message("Failed to find todo for editing".to_string());
                    }
                } else {
                    self.set_message("No workspace selected".to_string());
                }
            } else {
                self.set_message("Cannot save empty todo description".to_string());
            }
        }
        self.exit_edit_mode();
    }
    
    pub fn exit_edit_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.edit_buffer.clear();
        self.edit_cursor_pos = 0;
        self.editing_todo_id = None;
    }
    
    pub fn add_char_to_edit(&mut self, c: char) {
        self.edit_buffer.insert(self.edit_cursor_pos, c);
        self.edit_cursor_pos += c.len_utf8();
    }
    
    pub fn remove_char_from_edit(&mut self) {
        if self.edit_cursor_pos > 0 {
            // Find the start of the character to remove (handle UTF-8)
            let mut char_start = self.edit_cursor_pos - 1;
            while char_start > 0 && !self.edit_buffer.is_char_boundary(char_start) {
                char_start -= 1;
            }
            
            self.edit_buffer.remove(char_start);
            self.edit_cursor_pos = char_start;
        }
    }
    
    // Input buffer character manipulation
    pub fn add_char_to_input(&mut self, c: char) {
        self.input_buffer.insert(self.input_cursor_pos, c);
        self.input_cursor_pos += c.len_utf8();
    }
    
    pub fn remove_char_from_input(&mut self) {
        if self.input_cursor_pos > 0 {
            // Find the start of the character to remove (handle UTF-8)
            let mut char_start = self.input_cursor_pos - 1;
            while char_start > 0 && !self.input_buffer.is_char_boundary(char_start) {
                char_start -= 1;
            }
            
            self.input_buffer.remove(char_start);
            self.input_cursor_pos = char_start;
        }
    }
    
    // Cursor navigation for notes
    pub fn move_notes_cursor_left(&mut self) {
        if self.notes_cursor_pos > 0 {
            self.notes_cursor_pos -= 1;
            // Ensure we're at a valid character boundary
            while self.notes_cursor_pos > 0 && !self.notes_buffer.is_char_boundary(self.notes_cursor_pos) {
                self.notes_cursor_pos -= 1;
            }
        }
    }
    
    pub fn move_notes_cursor_right(&mut self) {
        if self.notes_cursor_pos < self.notes_buffer.len() {
            self.notes_cursor_pos += 1;
            // Ensure we're at a valid character boundary
            while self.notes_cursor_pos < self.notes_buffer.len() && !self.notes_buffer.is_char_boundary(self.notes_cursor_pos) {
                self.notes_cursor_pos += 1;
            }
        }
    }
    
    // Cursor navigation for edit buffer
    pub fn move_edit_cursor_left(&mut self) {
        if self.edit_cursor_pos > 0 {
            self.edit_cursor_pos -= 1;
            // Ensure we're at a valid character boundary
            while self.edit_cursor_pos > 0 && !self.edit_buffer.is_char_boundary(self.edit_cursor_pos) {
                self.edit_cursor_pos -= 1;
            }
        }
    }
    
    pub fn move_edit_cursor_right(&mut self) {
        if self.edit_cursor_pos < self.edit_buffer.len() {
            self.edit_cursor_pos += 1;
            // Ensure we're at a valid character boundary
            while self.edit_cursor_pos < self.edit_buffer.len() && !self.edit_buffer.is_char_boundary(self.edit_cursor_pos) {
                self.edit_cursor_pos += 1;
            }
        }
    }
    
    // Cursor navigation for input buffer
    pub fn move_input_cursor_left(&mut self) {
        if self.input_cursor_pos > 0 {
            self.input_cursor_pos -= 1;
            // Ensure we're at a valid character boundary
            while self.input_cursor_pos > 0 && !self.input_buffer.is_char_boundary(self.input_cursor_pos) {
                self.input_cursor_pos -= 1;
            }
        }
    }
    
    pub fn move_input_cursor_right(&mut self) {
        if self.input_cursor_pos < self.input_buffer.len() {
            self.input_cursor_pos += 1;
            // Ensure we're at a valid character boundary
            while self.input_cursor_pos < self.input_buffer.len() && !self.input_buffer.is_char_boundary(self.input_cursor_pos) {
                self.input_cursor_pos += 1;
            }
        }
    }
    
    // Clear input buffer and reset cursor
    pub fn clear_input_buffer(&mut self) {
        self.input_buffer.clear();
        self.input_cursor_pos = 0;
    }
    
    // Welcome screen methods
    pub fn get_welcome_options(&self) -> Vec<(&str, &str)> {
        if self.is_first_launch {
            // First time user options
            vec![
                ("🚀 Get Started", "Create your first todo and jump right in"),
                ("📂 Browse Workspaces", "Explore existing workspaces or create new ones"),
                ("❓ Learn the Basics", "View help and keyboard shortcuts"),
                ("⚡ Quick Demo", "See Paperclip in action with sample todos"),
                ("❌ Exit", "Close Paperclip"),
            ]
        } else {
            // Returning user options  
            vec![
                ("📂 Browse Workspaces", "Select from your existing workspaces"),
                ("❓ Learn the Basics", "View help and keyboard shortcuts"),
                ("⚡ Quick Demo", "See Paperclip in action with sample todos"),
                ("🆕 Create New Workspace", "Start fresh with a new workspace"),
                ("❌ Exit", "Close Paperclip"),
            ]
        }
    }
    
    pub fn move_welcome_selection_up(&mut self) {
        if self.welcome_selected > 0 {
            self.welcome_selected -= 1;
        }
    }
    
    pub fn move_welcome_selection_down(&mut self) {
        let max_options = self.get_welcome_options().len();
        if self.welcome_selected < max_options.saturating_sub(1) {
            self.welcome_selected += 1;
        }
    }
    
    pub fn select_welcome_option(&mut self) {
        if self.is_first_launch {
            // First time user options: Get Started | Browse Workspaces | Learn | Demo | Exit
            match self.welcome_selected {
                0 => {
                    // Get Started - create Personal workspace and go to insert mode
                    self.workspace_manager.switch_workspace_by_name("Personal");
                    self.mode = AppMode::Insert;
                    self.clear_input_buffer();
                    self.set_message("Welcome! Type your first todo and press Enter".to_string());
                }
                1 => {
                    // Browse Workspaces
                    self.enter_workspace_selection();
                }
                2 => {
                    // Learn the Basics
                    self.show_help = true;
                    self.set_message("Press ? again to close help".to_string());
                }
                3 => {
                    // Quick Demo
                    self.create_demo_todos();
                    self.workspace_manager.switch_workspace_by_name("Personal");
                    self.mode = AppMode::Normal;
                    self.set_message("Welcome! Try navigating with j/k, press Space to complete todos".to_string());
                }
                4 => {
                    // Exit
                    self.should_quit = true;
                }
                _ => {}
            }
        } else {
            // Returning user options: Browse Workspaces | Learn | Demo | Create New | Exit
            match self.welcome_selected {
                0 => {
                    // Browse Workspaces
                    self.enter_workspace_selection();
                }
                1 => {
                    // Learn the Basics
                    self.show_help = true;
                    self.set_message("Press ? again to close help".to_string());
                }
                2 => {
                    // Quick Demo
                    self.create_demo_todos();
                    self.workspace_manager.switch_workspace_by_name("Personal");
                    self.mode = AppMode::Normal;
                    self.set_message("Welcome! Try navigating with j/k, press Space to complete todos".to_string());
                }
                3 => {
                    // Create New Workspace
                    self.enter_create_workspace_mode();
                }
                4 => {
                    // Exit
                    self.should_quit = true;
                }
                _ => {}
            }
        }
    }
    
    // Return to welcome screen from any mode
    pub fn return_to_welcome(&mut self) {
        self.mode = AppMode::Welcome;
        self.welcome_selected = 0;
        self.selected = 0;
        self.clear_input_buffer();
        self.set_message("Returned to welcome screen - Choose an option to continue".to_string());
    }
    
    fn create_demo_todos(&mut self) {
        if let Some(todo_list) = self.get_current_todo_list_mut() {
            // Clear existing todos in Personal workspace for demo
            todo_list.todos.clear();
            
            // Add demo todos
            let demo_todos = vec![
                "Welcome to Paperclip! #getting-started @demo due:today",
                "Try pressing Space to mark this todo complete ○ #tutorial",
                "Press 'a' on this todo to add a child task ○ #tutorial",
                "Use j/k or arrow keys to navigate ○ #navigation",
                "Press 'i' to add a new todo ○ #basics",
                "Press 'n' to add notes to a todo ○ #features",
                "Try searching with '/' key ○ #search",
                "Filter by tags with '#' key #important @work due:tomorrow",
                "Press '?' for help anytime ○ #help",
            ];
            
            for todo_text in demo_todos {
                todo_list.add_todo(todo_text.to_string());
            }
        }
    }
    
    // Template management
    pub fn enter_template_selection(&mut self) {
        let templates = self.template_manager.get_all_templates();
        self.available_templates = templates.iter().map(|t| t.id.clone()).collect();
        
        if self.available_templates.is_empty() {
            self.set_message("No templates available".to_string());
            return;
        }
        
        self.mode = AppMode::TemplateSelection;
        self.popup_selected = 0;
    }
    
    pub fn apply_template(&mut self) {
        if let Some(todo_id) = self.get_selected_todo_id() {
            if let Some(template_id) = self.available_templates.get(self.popup_selected) {
                let template_id = template_id.clone();
                // Clone the template to avoid borrow checker issues
                if let Some(template) = self.template_manager.get_template(&template_id).cloned() {
                    let template_name = template.name.clone();
                    if let Some(todo_list) = self.get_current_todo_list_mut() {
                        if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                            template.apply_to_todo(todo);
                            self.set_message(format!("Applied template: {}", template_name));
                        }
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
        self.available_templates.clear();
    }
    
    // Recurrence pattern selection
    pub fn enter_recurrence_selection(&mut self) {
        self.mode = AppMode::RecurrenceSelection;
        self.popup_selected = 0;
    }
    
    pub fn apply_recurrence(&mut self) {
        if let Some(id) = self.get_selected_todo_id() {
            if let Some(pattern) = self.available_recurrence.get(self.popup_selected) {
                let pattern = pattern.clone();
                let pattern_name = match &pattern {
                    RecurrencePattern::None => "None",
                    RecurrencePattern::Daily => "Daily",
                    RecurrencePattern::Weekly => "Weekly", 
                    RecurrencePattern::Monthly => "Monthly",
                    RecurrencePattern::Yearly => "Yearly",
                    RecurrencePattern::Custom(_days) => "Custom",
                };
                
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(id) {
                        todo.set_recurrence(pattern);
                        self.set_message(format!("Recurrence set to: {}", pattern_name));
                    }
                }
            }
        }
        self.mode = AppMode::Normal;
    }
    
    // Process recurring todos (call this periodically)
    pub fn update_recurring_todos(&mut self) {
        if let Some(todo_list) = self.get_current_todo_list_mut() {
            let old_count = todo_list.total_count();
            todo_list.process_recurring_todos();
            let new_count = todo_list.total_count();
            
            if new_count > old_count {
                self.set_message(format!("Generated {} recurring todos", new_count - old_count));
            }
        }
    }
    
    // Workspace management methods
    pub fn enter_workspace_selection(&mut self) {
        let workspace_names = self.workspace_manager.get_all_workspaces()
            .iter()
            .map(|ws| ws.name.clone())
            .collect();
        
        self.available_workspaces = workspace_names;
        
        if self.available_workspaces.is_empty() {
            self.set_message("No workspaces available".to_string());
            return;
        }
        
        self.mode = AppMode::WorkspaceSelection;
        self.popup_selected = 0;
    }
    
    pub fn switch_workspace(&mut self) {
        if self.popup_selected == 0 {
            // Home option selected - return to welcome screen
            self.return_to_welcome();
        } else {
            // Regular workspace selection (subtract 1 to account for Home option)
            let workspace_index = self.popup_selected - 1;
            if let Some(workspace_name) = self.available_workspaces.get(workspace_index) {
                if self.workspace_manager.switch_workspace_by_name(workspace_name) {
                    self.set_message(format!("Switched to workspace: {}", workspace_name));
                    self.selected = 0; // Reset selection when switching workspaces
                    self.view_mode = ViewMode::All; // Reset view mode
                    self.mode = AppMode::Normal;
                } else {
                    self.set_message("Failed to switch workspace".to_string());
                }
            }
        }
        self.available_workspaces.clear();
    }
    
    pub fn create_new_workspace(&mut self, name: String, description: Option<String>) {
        let workspace_id = self.workspace_manager.create_workspace(name.clone(), description);
        self.set_message(format!("Created workspace: {} (ID: {})", name, workspace_id));
    }
    
    pub fn delete_current_workspace(&mut self) {
        let current_name = self.get_current_workspace_name();
        if self.workspace_manager.delete_workspace(&current_name) {
            self.set_message(format!("Deleted workspace: {}", current_name));
            self.selected = 0; // Reset selection
            self.view_mode = ViewMode::All; // Reset view mode
        } else {
            self.set_message("Cannot delete the last remaining workspace".to_string());
        }
    }
    
    pub fn rename_current_workspace(&mut self, new_name: String) {
        let current_name = self.get_current_workspace_name();
        if self.workspace_manager.rename_workspace(&current_name, new_name.clone()) {
            self.set_message(format!("Renamed workspace to: {}", new_name));
        } else {
            self.set_message("Failed to rename workspace (name may already exist)".to_string());
        }
    }
    
    // Workspace creation
    pub fn enter_create_workspace_mode(&mut self) {
        self.mode = AppMode::CreateWorkspace;
        self.input_buffer.clear();
        self.set_message("Enter workspace name:".to_string());
    }
    
    pub fn submit_workspace_creation(&mut self) {
        if !self.input_buffer.trim().is_empty() {
            let workspace_name = self.input_buffer.trim().to_string();
            
            // Check if workspace name already exists
            if self.workspace_manager.get_all_workspaces()
                .iter()
                .any(|ws| ws.name == workspace_name) {
                self.set_message("Workspace with this name already exists".to_string());
                return;
            }
            
            // Create the workspace
            let workspace_id = self.workspace_manager.create_workspace(
                workspace_name.clone(), 
                Some(format!("Workspace created by user"))
            );
            
            // Refresh available workspaces list
            self.available_workspaces = self.workspace_manager.get_all_workspaces()
                .iter()
                .map(|ws| ws.name.clone())
                .collect();
            
            // Switch to the newly created workspace
            if self.workspace_manager.switch_workspace_by_name(&workspace_name) {
                self.set_message(format!("Created and switched to workspace: {}", workspace_name));
                self.mode = AppMode::Normal;
                self.selected = 0;
                self.view_mode = ViewMode::All;
            } else {
                self.set_message(format!("Created workspace: {} (ID: {}), but failed to switch", workspace_name, workspace_id));
                self.mode = AppMode::WorkspaceSelection;
            }
        } else {
            self.set_message("Workspace name cannot be empty".to_string());
        }
        self.input_buffer.clear();
    }
    
    pub fn cancel_workspace_creation(&mut self) {
        self.mode = AppMode::WorkspaceSelection;
        self.input_buffer.clear();
        self.set_message("Workspace creation cancelled".to_string());
    }
    
    pub fn delete_selected_workspace(&mut self) {
        if self.popup_selected == 0 {
            // Can't delete the Home option
            self.set_message("Cannot delete the Home option".to_string());
            return;
        }
        
        // Adjust index to account for Home option
        let workspace_index = self.popup_selected - 1;
        if let Some(workspace_name) = self.available_workspaces.get(workspace_index) {
            // Find workspace ID by name
            if let Some((workspace_id, _)) = self.workspace_manager.workspaces.iter().find(|(_, ws)| ws.name == *workspace_name) {
                let workspace_id = workspace_id.clone();
                if self.workspace_manager.delete_workspace(&workspace_id) {
                    self.set_message(format!("Deleted workspace: {}", workspace_name));
                    
                    // Refresh available workspaces list
                    self.available_workspaces = self.workspace_manager.get_all_workspaces()
                        .iter()
                        .map(|ws| ws.name.clone())
                        .collect();
                    
                    // Adjust popup selection if needed
                    if self.popup_selected >= self.available_workspaces.len() && !self.available_workspaces.is_empty() {
                        self.popup_selected = self.available_workspaces.len() - 1;
                    }
                    
                    // If no workspaces left, exit to normal mode
                    if self.available_workspaces.is_empty() {
                        self.mode = AppMode::Normal;
                        self.set_message("All workspaces deleted. Creating default workspace.".to_string());
                        // Create a default workspace
                        self.workspace_manager.create_workspace("Personal".to_string(), Some("Default workspace".to_string()));
                    }
                } else {
                    self.set_message("Cannot delete the last remaining workspace".to_string());
                }
            }
        }
    }
    
    // Undo/Redo functionality
    pub fn undo(&mut self) {
        if let Some(command) = self.command_history.undo() {
            self.execute_undo_command(command);
        } else {
            self.set_message("Nothing to undo".to_string());
        }
    }
    
    pub fn redo(&mut self) {
        if let Some(command) = self.command_history.redo() {
            self.execute_redo_command(command);
        } else {
            self.set_message("Nothing to redo".to_string());
        }
    }
    
    fn execute_undo_command(&mut self, command: Command) {
        match command {
            Command::AddTodo { workspace_id: _workspace_id, todo } => {
                // Undo add: remove the todo
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    todo_list.remove_todo(todo.id);
                    self.set_message(format!("Undid: Add todo '{}'", todo.description));
                }
            },
            Command::DeleteTodo { workspace_id: _workspace_id, todo } => {
                // Undo delete: restore the todo
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    // Restore parent-child relationships if needed
                    if let Some(parent_id) = todo.parent_id {
                        if let Some(parent) = todo_list.get_todo_mut(parent_id) {
                            if !parent.children.contains(&todo.id) {
                                parent.children.push(todo.id);
                            }
                        }
                    }
                    todo_list.todos.insert(todo.id, todo.clone());
                    self.set_message(format!("Undid: Delete todo '{}'", todo.description));
                }
            },
            Command::CompleteTodo { workspace_id: _workspace_id, todo_id, old_status } => {
                // Undo complete: restore old status
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                        todo.status = old_status.clone();
                        if matches!(old_status, crate::todo::TodoStatus::Completed) {
                            todo.completed_at = Some(chrono::Local::now());
                        } else {
                            todo.completed_at = None;
                        }
                        self.set_message("Undid: Toggle todo completion".to_string());
                    }
                }
            },
            Command::EditTodo { workspace_id: _workspace_id, todo_id, old_description, old_raw_description } => {
                // Undo edit: restore old description
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                        todo.description = old_description;
                        todo.raw_description = old_raw_description;
                        self.set_message("Undid: Edit todo".to_string());
                    }
                }
            },
            Command::ChangePriority { workspace_id: _workspace_id, todo_id, old_priority } => {
                // Undo priority change: restore old priority
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                        todo.priority = old_priority;
                        self.set_message(format!("Undid: Priority change (restored to {})", old_priority));
                    }
                }
            },
            Command::AddChildTodo { workspace_id: _workspace_id, parent_id, child_todo } => {
                // Undo add child: remove the child todo
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    // Remove from parent's children list
                    if let Some(parent) = todo_list.get_todo_mut(parent_id) {
                        parent.children.retain(|&id| id != child_todo.id);
                    }
                    todo_list.remove_todo(child_todo.id);
                    self.set_message(format!("Undid: Add child todo '{}'", child_todo.description));
                }
            },
            Command::DeleteWithChildren { workspace_id: _workspace_id, deleted_todos } => {
                // Undo delete with children: restore all todos
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    // Restore all todos
                    for todo in &deleted_todos {
                        todo_list.todos.insert(todo.id, todo.clone());
                    }
                    // Restore parent-child relationships
                    for todo in &deleted_todos {
                        if let Some(parent_id) = todo.parent_id {
                            if let Some(parent) = todo_list.get_todo_mut(parent_id) {
                                if !parent.children.contains(&todo.id) {
                                    parent.children.push(todo.id);
                                }
                            }
                        }
                    }
                    self.set_message(format!("Undid: Delete {} todos with children", deleted_todos.len()));
                }
            },
        }
    }
    
    fn execute_redo_command(&mut self, command: Command) {
        // Redo is essentially re-executing the original command
        match command {
            Command::AddTodo { workspace_id: _workspace_id, todo } => {
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    todo_list.todos.insert(todo.id, todo.clone());
                    self.set_message(format!("Redid: Add todo '{}'", todo.description));
                }
            },
            Command::DeleteTodo { workspace_id: _workspace_id, todo } => {
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    todo_list.remove_todo(todo.id);
                    self.set_message(format!("Redid: Delete todo '{}'", todo.description));
                }
            },
            Command::CompleteTodo { workspace_id: _workspace_id, todo_id, old_status: _old_status } => {
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                        todo.toggle_complete();
                        let status = if todo.is_completed() { "completed" } else { "pending" };
                        self.set_message(format!("Redid: Todo marked as {}", status));
                    }
                }
            },
            Command::ChangePriority { workspace_id: _workspace_id, todo_id, old_priority } => {
                // For redo, we need to toggle the priority back
                if let Some(todo_list) = self.get_current_todo_list_mut() {
                    if let Some(todo) = todo_list.get_todo_mut(todo_id) {
                        let current_priority = todo.priority;
                        todo.priority = old_priority;
                        self.set_message(format!("Redid: Priority change (from {} to {})", current_priority, old_priority));
                    }
                }
            },
            _ => {
                self.set_message("Redo operation not fully implemented for this command type".to_string());
            }
        }
    }
}
