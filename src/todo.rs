use chrono::{DateTime, Local, NaiveDate, Datelike, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TodoStatus {
    Pending,
    Completed,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecurrencePattern {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Custom(u32), // Custom interval in days
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub start: DateTime<Local>,
    pub end: Option<DateTime<Local>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeTracker {
    pub total_seconds: u64,
    pub entries: Vec<TimeEntry>,
    pub current_session: Option<DateTime<Local>>, // When current session started
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub description: String,
    pub raw_description: String, // Original input with tags
    pub tags: HashSet<String>,   // #tags extracted from description
    pub contexts: HashSet<String>, // @contexts extracted from description
    pub status: TodoStatus,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub due_date: Option<DateTime<Local>>,
    pub priority: u8, // 0-5, higher is more important
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
    pub expanded: bool, // For UI - whether children are shown
    
    // Advanced features
    pub notes: Option<String>, // Detailed notes/description
    pub time_tracker: TimeTracker, // Time tracking data
    pub recurrence: RecurrencePattern, // Recurring pattern
    pub template_id: Option<String>, // If created from template
}

impl Todo {
    pub fn new(id: u32, raw_description: String) -> Self {
        let (clean_description, tags, contexts, due_date) = Self::parse_description(&raw_description);
        
        Self {
            id,
            description: clean_description,
            raw_description,
            tags,
            contexts,
            status: TodoStatus::Pending,
            created_at: Local::now(),
            completed_at: None,
            due_date,
            priority: 0,
            parent_id: None,
            children: Vec::new(),
            expanded: true,
            
            // Initialize advanced features
            notes: None,
            time_tracker: TimeTracker {
                total_seconds: 0,
                entries: Vec::new(),
                current_session: None,
            },
            recurrence: RecurrencePattern::None,
            template_id: None,
        }
    }
    
    fn parse_description(input: &str) -> (String, HashSet<String>, HashSet<String>, Option<DateTime<Local>>) {
        let mut description = input.to_string();
        let mut tags = HashSet::new();
        let mut contexts = HashSet::new();
        let mut due_date = None;
        
        // Extract #tags
        let tag_re = Regex::new(r"#([a-zA-Z0-9_]+)").unwrap();
        for cap in tag_re.captures_iter(input) {
            if let Some(tag) = cap.get(1) {
                tags.insert(tag.as_str().to_lowercase());
            }
        }
        
        // Extract @contexts
        let context_re = Regex::new(r"@([a-zA-Z0-9_]+)").unwrap();
        for cap in context_re.captures_iter(input) {
            if let Some(context) = cap.get(1) {
                contexts.insert(context.as_str().to_lowercase());
            }
        }
        
        // Extract due dates - simple patterns for now
        let due_re = Regex::new(r"due:([\w\-/]+)").unwrap();
        if let Some(cap) = due_re.captures(input) {
            if let Some(due_str) = cap.get(1) {
                due_date = Self::parse_due_date(due_str.as_str());
                description = due_re.replace(&description, "").to_string();
            }
        }
        
        // Clean up description by removing tag/context markers but keeping the words
        description = tag_re.replace_all(&description, "$1").to_string();
        description = context_re.replace_all(&description, "$1").to_string();
        description = description.trim().to_string();
        
        (description, tags, contexts, due_date)
    }
    
    fn parse_due_date(date_str: &str) -> Option<DateTime<Local>> {
        let now = Local::now();
        
        match date_str.to_lowercase().as_str() {
            "today" => Some(now.date_naive().and_hms_opt(23, 59, 59)?.and_local_timezone(Local).single()?),
            "tomorrow" => Some((now.date_naive() + chrono::Duration::days(1)).and_hms_opt(23, 59, 59)?.and_local_timezone(Local).single()?),
            "monday" | "mon" => Some(Self::next_weekday(now, chrono::Weekday::Mon)),
            "tuesday" | "tue" => Some(Self::next_weekday(now, chrono::Weekday::Tue)),
            "wednesday" | "wed" => Some(Self::next_weekday(now, chrono::Weekday::Wed)),
            "thursday" | "thu" => Some(Self::next_weekday(now, chrono::Weekday::Thu)),
            "friday" | "fri" => Some(Self::next_weekday(now, chrono::Weekday::Fri)),
            "saturday" | "sat" => Some(Self::next_weekday(now, chrono::Weekday::Sat)),
            "sunday" | "sun" => Some(Self::next_weekday(now, chrono::Weekday::Sun)),
            _ => {
                // Try parsing YYYY-MM-DD format
                if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    naive_date.and_hms_opt(23, 59, 59)?.and_local_timezone(Local).single()
                } else {
                    None
                }
            }
        }
    }
    
    fn next_weekday(from: DateTime<Local>, target_weekday: chrono::Weekday) -> DateTime<Local> {
        let days_ahead = (target_weekday.number_from_monday() as i32 - from.weekday().number_from_monday() as i32 + 7) % 7;
        let days_ahead = if days_ahead == 0 { 7 } else { days_ahead }; // If it's today, go to next week
        
        (from.date_naive() + chrono::Duration::days(days_ahead as i64))
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(Local)
            .single()
            .unwrap()
    }

    pub fn complete(&mut self) {
        self.status = TodoStatus::Completed;
        self.completed_at = Some(Local::now());
    }

    pub fn uncomplete(&mut self) {
        self.status = TodoStatus::Pending;
        self.completed_at = None;
    }

    pub fn toggle_complete(&mut self) {
        match self.status {
            TodoStatus::Completed => self.uncomplete(),
            _ => self.complete(),
        }
    }

    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority.min(5);
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, TodoStatus::Completed)
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            due < Local::now() && !self.is_completed()
        } else {
            false
        }
    }
    
    // Time tracking methods
    pub fn start_timer(&mut self) {
        if self.time_tracker.current_session.is_none() {
            self.time_tracker.current_session = Some(Local::now());
            if self.status == TodoStatus::Pending {
                self.status = TodoStatus::InProgress;
            }
        }
    }
    
    pub fn stop_timer(&mut self) {
        if let Some(start_time) = self.time_tracker.current_session.take() {
            let end_time = Local::now();
            let duration = end_time.signed_duration_since(start_time);
            
            self.time_tracker.total_seconds += duration.num_seconds() as u64;
            self.time_tracker.entries.push(TimeEntry {
                start: start_time,
                end: Some(end_time),
                description: None,
            });
        }
    }
    
    pub fn is_timer_running(&self) -> bool {
        self.time_tracker.current_session.is_some()
    }
    
    pub fn get_current_session_duration(&self) -> Option<Duration> {
        self.time_tracker.current_session.map(|start| {
            Local::now().signed_duration_since(start)
        })
    }
    
    pub fn get_total_time_formatted(&self) -> String {
        let mut total_seconds = self.time_tracker.total_seconds;
        
        // Add current session time if running
        if let Some(duration) = self.get_current_session_duration() {
            total_seconds += duration.num_seconds() as u64;
        }
        
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
    
    // Notes methods
    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
    
    pub fn update_description(&mut self, new_raw_description: String) {
        let (clean_description, tags, contexts, due_date) = Self::parse_description(&new_raw_description);
        self.raw_description = new_raw_description;
        self.description = clean_description;
        self.tags = tags;
        self.contexts = contexts;
        self.due_date = due_date; // Always update due_date, even if None (to clear existing dates)
    }
    
    pub fn has_notes(&self) -> bool {
        self.notes.is_some() && !self.notes.as_ref().unwrap().trim().is_empty()
    }
    
    // Recurrence methods
    pub fn set_recurrence(&mut self, pattern: RecurrencePattern) {
        self.recurrence = pattern;
    }
    
    pub fn is_recurring(&self) -> bool {
        !matches!(self.recurrence, RecurrencePattern::None)
    }
    
    pub fn should_generate_next(&self) -> bool {
        self.is_completed() && self.is_recurring()
    }
    
    pub fn get_next_due_date(&self) -> Option<DateTime<Local>> {
        if let Some(current_due) = self.due_date {
            match self.recurrence {
                RecurrencePattern::Daily => Some(current_due + Duration::days(1)),
                RecurrencePattern::Weekly => Some(current_due + Duration::weeks(1)),
                RecurrencePattern::Monthly => {
                    // Add one month
                    let next_month = if current_due.month() == 12 {
                        current_due.with_year(current_due.year() + 1)?.with_month(1)?
                    } else {
                        current_due.with_month(current_due.month() + 1)?
                    };
                    Some(next_month)
                }
                RecurrencePattern::Yearly => Some(current_due.with_year(current_due.year() + 1)?),
                RecurrencePattern::Custom(days) => Some(current_due + Duration::days(days as i64)),
                RecurrencePattern::None => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub todos: HashMap<u32, Todo>,
    pub next_id: u32,
}

impl TodoList {
    pub fn new() -> Self {
        Self {
            todos: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_todo(&mut self, description: String) -> u32 {
        let id = self.next_id;
        let todo = Todo::new(id, description);
        self.todos.insert(id, todo);
        self.next_id += 1;
        id
    }

    pub fn remove_todo(&mut self, id: u32) -> Option<Todo> {
        // First, get the todo to check if it has a parent
        let todo = self.todos.get(&id);
        let parent_id = todo.and_then(|t| t.parent_id);
        
        // Remove from parent's children list if this todo has a parent
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.todos.get_mut(&parent_id) {
                parent.children.retain(|&child_id| child_id != id);
            }
        }
        
        // Remove the todo itself
        self.todos.remove(&id)
    }

    pub fn get_todo(&self, id: u32) -> Option<&Todo> {
        self.todos.get(&id)
    }

    pub fn get_todo_mut(&mut self, id: u32) -> Option<&mut Todo> {
        self.todos.get_mut(&id)
    }

    pub fn get_all_todos(&self) -> Vec<&Todo> {
        let mut todos: Vec<&Todo> = self.todos.values().collect();
        // Sort by priority (high to low), then by creation date
        todos.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        todos
    }

    pub fn get_pending_todos(&self) -> Vec<&Todo> {
        self.get_all_todos().into_iter()
            .filter(|todo| !todo.is_completed())
            .collect()
    }

    pub fn get_completed_todos(&self) -> Vec<&Todo> {
        self.get_all_todos().into_iter()
            .filter(|todo| todo.is_completed())
            .collect()
    }

    pub fn clear_completed(&mut self) {
        self.todos.retain(|_, todo| !todo.is_completed());
    }

    pub fn total_count(&self) -> usize {
        self.todos.len()
    }

    pub fn pending_count(&self) -> usize {
        self.todos.values().filter(|todo| !todo.is_completed()).count()
    }

    pub fn completed_count(&self) -> usize {
        self.todos.values().filter(|todo| todo.is_completed()).count()
    }

    // Hierarchical methods
    pub fn add_child_todo(&mut self, parent_id: u32, description: String) -> Option<u32> {
        // Check if parent exists
        if !self.todos.contains_key(&parent_id) {
            return None;
        }

        let child_id = self.next_id;
        let mut child_todo = Todo::new(child_id, description);
        child_todo.parent_id = Some(parent_id);
        
        // Add child to parent's children list
        if let Some(parent) = self.todos.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
        
        self.todos.insert(child_id, child_todo);
        self.next_id += 1;
        Some(child_id)
    }

    pub fn get_root_todos(&self) -> Vec<&Todo> {
        let mut todos: Vec<&Todo> = self.todos.values()
            .filter(|todo| todo.parent_id.is_none())
            .collect();
        
        // Sort by priority (high to low), then by creation date
        todos.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        todos
    }

    pub fn get_children(&self, parent_id: u32) -> Vec<&Todo> {
        if let Some(parent) = self.todos.get(&parent_id) {
            let mut children: Vec<&Todo> = parent.children.iter()
                .filter_map(|&child_id| self.todos.get(&child_id))
                .collect();
            
            // Sort children by priority, then creation date
            children.sort_by(|a, b| {
                b.priority.cmp(&a.priority)
                    .then_with(|| a.created_at.cmp(&b.created_at))
            });
            children
        } else {
            Vec::new()
        }
    }

    pub fn get_flattened_todos(&self) -> Vec<(&Todo, u32)> {
        let mut result = Vec::new();
        
        fn add_todo_and_children<'a>(
            todos: &'a HashMap<u32, Todo>,
            result: &mut Vec<(&'a Todo, u32)>,
            todo: &'a Todo,
            depth: u32,
        ) {
            result.push((todo, depth));
            
            if todo.expanded {
                let mut children: Vec<&Todo> = todo.children.iter()
                    .filter_map(|&child_id| todos.get(&child_id))
                    .collect();
                
                // Sort children
                children.sort_by(|a, b| {
                    b.priority.cmp(&a.priority)
                        .then_with(|| a.created_at.cmp(&b.created_at))
                });
                
                for child in children {
                    add_todo_and_children(todos, result, child, depth + 1);
                }
            }
        }
        
        let root_todos = self.get_root_todos();
        for todo in root_todos {
            add_todo_and_children(&self.todos, &mut result, todo, 0);
        }
        
        result
    }

    pub fn get_flattened_pending_todos(&self) -> Vec<(&Todo, u32)> {
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| !todo.is_completed())
            .collect()
    }

    pub fn get_flattened_completed_todos(&self) -> Vec<(&Todo, u32)> {
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| todo.is_completed())
            .collect()
    }

    pub fn toggle_expanded(&mut self, id: u32) {
        if let Some(todo) = self.todos.get_mut(&id) {
            todo.expanded = !todo.expanded;
        }
    }

    pub fn remove_todo_and_children(&mut self, id: u32) -> Vec<Todo> {
        let mut removed = Vec::new();
        
        // Get all children recursively
        fn collect_children(todos: &HashMap<u32, Todo>, parent_id: u32, collected: &mut Vec<u32>) {
            if let Some(parent) = todos.get(&parent_id) {
                for &child_id in &parent.children {
                    collected.push(child_id);
                    collect_children(todos, child_id, collected);
                }
            }
        }
        
        let mut to_remove = vec![id];
        collect_children(&self.todos, id, &mut to_remove);
        
        // Remove from parent's children list if this todo has a parent
        if let Some(todo) = self.todos.get(&id) {
            if let Some(parent_id) = todo.parent_id {
                if let Some(parent) = self.todos.get_mut(&parent_id) {
                    parent.children.retain(|&child_id| child_id != id);
                }
            }
        }
        
        // Remove all collected todos
        for todo_id in to_remove {
            if let Some(todo) = self.todos.remove(&todo_id) {
                removed.push(todo);
            }
        }
        
        removed
    }

    pub fn has_children(&self, id: u32) -> bool {
        self.todos.get(&id)
            .map(|todo| !todo.children.is_empty())
            .unwrap_or(false)
    }

    pub fn get_depth(&self, id: u32) -> u32 {
        let mut depth = 0;
        let mut current_id = id;
        
        while let Some(todo) = self.todos.get(&current_id) {
            if let Some(parent_id) = todo.parent_id {
                depth += 1;
                current_id = parent_id;
            } else {
                break;
            }
        }
        
        depth
    }
    
    // Filtering and search methods
    pub fn search_todos(&self, query: &str) -> Vec<(&Todo, u32)> {
        let query_lower = query.to_lowercase();
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| {
                todo.description.to_lowercase().contains(&query_lower) ||
                todo.tags.iter().any(|tag| tag.contains(&query_lower)) ||
                todo.contexts.iter().any(|ctx| ctx.contains(&query_lower))
            })
            .collect()
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<(&Todo, u32)> {
        let tag_lower = tag.to_lowercase();
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| todo.tags.contains(&tag_lower))
            .collect()
    }
    
    pub fn filter_by_context(&self, context: &str) -> Vec<(&Todo, u32)> {
        let context_lower = context.to_lowercase();
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| todo.contexts.contains(&context_lower))
            .collect()
    }
    
    pub fn filter_by_due_date(&self, filter_type: DueDateFilter) -> Vec<(&Todo, u32)> {
        let now = Local::now();
        let today = now.date_naive();
        
        self.get_flattened_todos().into_iter()
            .filter(|(todo, _)| {
                match (&todo.due_date, filter_type) {
                    (Some(due), DueDateFilter::Overdue) => due < &now && !todo.is_completed(),
                    (Some(due), DueDateFilter::Today) => due.date_naive() == today,
                    (Some(due), DueDateFilter::Tomorrow) => due.date_naive() == today + chrono::Duration::days(1),
                    (Some(due), DueDateFilter::ThisWeek) => {
                        let week_from_now = now + chrono::Duration::days(7);
                        due >= &now && due <= &week_from_now
                    },
                    (None, DueDateFilter::NoDueDate) => true,
                    _ => false,
                }
            })
            .collect()
    }
    
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: HashSet<String> = HashSet::new();
        for todo in self.todos.values() {
            tags.extend(todo.tags.iter().cloned());
        }
        let mut sorted_tags: Vec<String> = tags.into_iter().collect();
        sorted_tags.sort();
        sorted_tags
    }
    
    pub fn get_all_contexts(&self) -> Vec<String> {
        let mut contexts: HashSet<String> = HashSet::new();
        for todo in self.todos.values() {
            contexts.extend(todo.contexts.iter().cloned());
        }
        let mut sorted_contexts: Vec<String> = contexts.into_iter().collect();
        sorted_contexts.sort();
        sorted_contexts
    }
    
    pub fn get_overdue_count(&self) -> usize {
        self.todos.values().filter(|todo| todo.is_overdue()).count()
    }
    
    pub fn get_due_today_count(&self) -> usize {
        let today = Local::now().date_naive();
        self.todos.values().filter(|todo| {
            if let Some(due) = todo.due_date {
                due.date_naive() == today && !todo.is_completed()
            } else {
                false
            }
        }).count()
    }
    
    pub fn get_tag_counts(&self) -> Vec<(String, usize)> {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for todo in self.todos.values() {
            for tag in &todo.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        let mut sorted_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        sorted_tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // Sort by count desc, then name asc
        sorted_tags
    }
    
    pub fn get_context_counts(&self) -> Vec<(String, usize)> {
        let mut context_counts: HashMap<String, usize> = HashMap::new();
        
        for todo in self.todos.values() {
            for context in &todo.contexts {
                *context_counts.entry(context.clone()).or_insert(0) += 1;
            }
        }
        
        let mut sorted_contexts: Vec<(String, usize)> = context_counts.into_iter().collect();
        sorted_contexts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))); // Sort by count desc, then name asc
        sorted_contexts
    }
    
    // Advanced feature methods
    
    // Recurring todos
    pub fn process_recurring_todos(&mut self) {
        let mut new_todos = Vec::new();
        
        for todo in self.todos.values() {
            if todo.should_generate_next() {
                if let Some(next_due) = todo.get_next_due_date() {
                    let mut new_todo = Todo::new(self.next_id, todo.raw_description.clone());
                    new_todo.due_date = Some(next_due);
                    new_todo.recurrence = todo.recurrence.clone();
                    new_todo.notes = todo.notes.clone();
                    new_todo.priority = todo.priority;
                    new_todo.tags = todo.tags.clone();
                    new_todo.contexts = todo.contexts.clone();
                    
                    new_todos.push(new_todo);
                    self.next_id += 1;
                }
            }
        }
        
        for todo in new_todos {
            self.todos.insert(todo.id, todo);
        }
    }
    
    // Time tracking helpers
    pub fn start_timer(&mut self, id: u32) {
        if let Some(todo) = self.todos.get_mut(&id) {
            todo.start_timer();
        }
    }
    
    pub fn stop_timer(&mut self, id: u32) {
        if let Some(todo) = self.todos.get_mut(&id) {
            todo.stop_timer();
        }
    }
    
    pub fn get_active_timers(&self) -> Vec<&Todo> {
        self.todos.values()
            .filter(|todo| todo.is_timer_running())
            .collect()
    }
    
    // Template-related methods will be added when we create the template system
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DueDateFilter {
    Overdue,
    Today,
    Tomorrow,
    ThisWeek,
    NoDueDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub color: Option<u8>, // Index into a predefined color palette
}

impl Workspace {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            created_at: Local::now(),
            color: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_color(mut self, color: u8) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManager {
    pub workspaces: HashMap<String, Workspace>,
    pub workspace_todos: HashMap<String, TodoList>,
    pub current_workspace: Option<String>,
    pub next_workspace_id: u32,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            workspace_todos: HashMap::new(),
            current_workspace: None,
            next_workspace_id: 1,
        }
    }
    
    pub fn create_workspace(&mut self, name: String, description: Option<String>) -> String {
        let id = format!("ws_{}", self.next_workspace_id);
        self.next_workspace_id += 1;
        
        let workspace = if let Some(desc) = description {
            Workspace::new(id.clone(), name).with_description(desc)
        } else {
            Workspace::new(id.clone(), name)
        };
        
        self.workspaces.insert(id.clone(), workspace);
        self.workspace_todos.insert(id.clone(), TodoList::new());
        
        // Set as current workspace if it's the first one
        if self.current_workspace.is_none() {
            self.current_workspace = Some(id.clone());
        }
        
        id
    }
    
    pub fn switch_workspace(&mut self, workspace_id: &str) -> bool {
        if self.workspaces.contains_key(workspace_id) {
            self.current_workspace = Some(workspace_id.to_string());
            true
        } else {
            false
        }
    }
    
    pub fn switch_workspace_by_name(&mut self, workspace_name: &str) -> bool {
        // Find workspace ID by name
        if let Some((workspace_id, _)) = self.workspaces.iter().find(|(_, ws)| ws.name == workspace_name) {
            let workspace_id = workspace_id.clone();
            self.current_workspace = Some(workspace_id);
            true
        } else {
            false
        }
    }
    
    pub fn get_current_workspace(&self) -> Option<&Workspace> {
        if let Some(current_id) = &self.current_workspace {
            self.workspaces.get(current_id)
        } else {
            None
        }
    }
    
    pub fn get_current_todo_list(&self) -> Option<&TodoList> {
        if let Some(current_id) = &self.current_workspace {
            self.workspace_todos.get(current_id)
        } else {
            None
        }
    }
    
    pub fn get_current_todo_list_mut(&mut self) -> Option<&mut TodoList> {
        if let Some(current_id) = &self.current_workspace {
            self.workspace_todos.get_mut(current_id)
        } else {
            None
        }
    }
    
    pub fn get_all_workspaces(&self) -> Vec<&Workspace> {
        let mut workspaces: Vec<&Workspace> = self.workspaces.values().collect();
        workspaces.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        workspaces
    }
    
    pub fn get_workspace_counts(&self) -> Vec<(String, String, usize)> {
        self.workspaces.values()
            .map(|workspace| {
                let todo_count = self.workspace_todos.get(&workspace.id)
                    .map(|todos| todos.total_count())
                    .unwrap_or(0);
                (workspace.id.clone(), workspace.name.clone(), todo_count)
            })
            .collect()
    }
    
    pub fn delete_workspace(&mut self, workspace_id: &str) -> bool {
        if self.workspaces.len() <= 1 {
            // Don't allow deleting the last workspace
            return false;
        }
        
        if self.workspaces.remove(workspace_id).is_some() {
            self.workspace_todos.remove(workspace_id);
            
            // If we deleted the current workspace, switch to another one
            if self.current_workspace.as_ref() == Some(&workspace_id.to_string()) {
                self.current_workspace = self.workspaces.keys().next().cloned();
            }
            
            true
        } else {
            false
        }
    }
    
    pub fn rename_workspace(&mut self, workspace_id: &str, new_name: String) -> bool {
        if let Some(workspace) = self.workspaces.get_mut(workspace_id) {
            workspace.name = new_name;
            true
        } else {
            false
        }
    }
    
    // Search across all workspaces
    pub fn search_all_workspaces(&self, query: &str) -> Vec<(String, Vec<(&Todo, u32)>)> {
        let mut results = Vec::new();
        
        for (workspace_id, todo_list) in &self.workspace_todos {
            let workspace_results = todo_list.search_todos(query);
            if !workspace_results.is_empty() {
                results.push((workspace_id.clone(), workspace_results));
            }
        }
        
        results
    }
    
    pub fn is_empty(&self) -> bool {
        self.workspaces.is_empty()
    }
    
    pub fn ensure_workspace(&mut self) -> String {
        if self.workspaces.is_empty() {
            self.create_workspace("Personal".to_string(), Some("Default workspace".to_string()))
        } else if self.current_workspace.is_none() {
            let first_id = self.workspaces.keys().next().unwrap().clone();
            self.current_workspace = Some(first_id.clone());
            first_id
        } else {
            self.current_workspace.clone().unwrap()
        }
    }
}

impl Default for TodoList {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}
