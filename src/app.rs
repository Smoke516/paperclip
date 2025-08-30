use crate::colors::TokyoNightColors;
use crate::todo::{Todo, TodoList, DueDateFilter, RecurrencePattern, WorkspaceManager};
use crate::template::TemplateManager;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
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

pub struct App {
    pub workspace_manager: WorkspaceManager,
    pub mode: AppMode,
    pub view_mode: ViewMode,
    pub selected: usize,
    pub input_buffer: String,
    pub search_buffer: String,
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
    pub editing_notes_for: Option<u32>, // Which todo's notes we're editing
    pub edit_buffer: String, // For editing todo descriptions
    pub editing_todo_id: Option<u32>, // Which todo's description we're editing
    pub available_templates: Vec<String>, // Template IDs for selection
    pub available_recurrence: Vec<RecurrencePattern>, // For recurrence selection
    
    // Workspace management
    pub available_workspaces: Vec<String>, // Workspace IDs for selection
}

impl App {
    pub fn new() -> Self {
        let mut workspace_manager = WorkspaceManager::new();
        // Create initial workspace
        workspace_manager.create_workspace("Personal".to_string(), Some("Your personal todos".to_string()));
        
        // Get available workspace names for initial selection
        let available_workspaces = workspace_manager.get_all_workspaces()
            .iter()
            .map(|ws| ws.name.clone())
            .collect();
        
        Self {
            workspace_manager,
            mode: AppMode::WorkspaceSelection,
            view_mode: ViewMode::All,
            selected: 0,
            input_buffer: String::new(),
            search_buffer: String::new(),
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
            editing_notes_for: None,
            edit_buffer: String::new(),
            editing_todo_id: None,
            available_templates: Vec::new(),
            available_recurrence: vec![
                RecurrencePattern::None,
                RecurrencePattern::Daily,
                RecurrencePattern::Weekly,
                RecurrencePattern::Monthly,
                RecurrencePattern::Yearly,
            ],
            available_workspaces,
        }
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
        self.input_buffer.clear();
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.inserting_child_for = None;
    }

    pub fn add_char_to_input(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn remove_char_from_input(&mut self) {
        self.input_buffer.pop();
    }

    pub fn submit_input(&mut self) {
        if !self.input_buffer.trim().is_empty() {
            let input_text = self.input_buffer.trim().to_string();
            match self.mode {
                AppMode::Insert => {
                    if let Some(todo_list) = self.get_current_todo_list_mut() {
                        todo_list.add_todo(input_text);
                        self.set_message("Todo added!".to_string());
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
            if let Some(todo_list) = self.get_current_todo_list_mut() {
                if let Some(todo) = todo_list.get_todo_mut(id) {
                    todo.toggle_complete();
                    let status = if todo.is_completed() { "completed" } else { "pending" };
                    self.set_message(format!("Todo marked as {}", status));
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
            self.input_buffer.clear();
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
    }

    pub fn add_char_to_search(&mut self, c: char) {
        self.search_buffer.push(c);
    }

    pub fn remove_char_from_search(&mut self) {
        self.search_buffer.pop();
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
            AppMode::WorkspaceSelection => self.available_workspaces.len(),
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
                } else {
                    self.notes_buffer.clear();
                }
            } else {
                self.notes_buffer.clear();
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
        self.notes_buffer.push(c);
    }
    
    pub fn remove_char_from_notes(&mut self) {
        self.notes_buffer.pop();
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
                } else {
                    self.edit_buffer.clear();
                }
            } else {
                self.edit_buffer.clear();
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
        self.editing_todo_id = None;
    }
    
    pub fn add_char_to_edit(&mut self, c: char) {
        self.edit_buffer.push(c);
    }
    
    pub fn remove_char_from_edit(&mut self) {
        self.edit_buffer.pop();
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
        if let Some(workspace_name) = self.available_workspaces.get(self.popup_selected) {
            if self.workspace_manager.switch_workspace_by_name(workspace_name) {
                self.set_message(format!("Switched to workspace: {}", workspace_name));
                self.selected = 0; // Reset selection when switching workspaces
                self.view_mode = ViewMode::All; // Reset view mode
            } else {
                self.set_message("Failed to switch workspace".to_string());
            }
        }
        self.mode = AppMode::Normal;
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
        if let Some(workspace_name) = self.available_workspaces.get(self.popup_selected) {
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
}
