use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Local};
use crate::todo::{Todo, RecurrencePattern};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub contexts: HashSet<String>,
    pub priority: u8,
    pub recurrence: RecurrencePattern,
    pub notes: Option<String>,
    pub created_at: DateTime<Local>,
    pub children: Vec<TodoTemplate>, // For template hierarchies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManager {
    pub templates: HashMap<String, TodoTemplate>,
}

impl TodoTemplate {
    pub fn new(name: String, description: String) -> Self {
        let template_id = format!("template_{}", Local::now().timestamp_millis());
        
        Self {
            id: template_id,
            name,
            description,
            tags: HashSet::new(),
            contexts: HashSet::new(),
            priority: 0,
            recurrence: RecurrencePattern::None,
            notes: None,
            created_at: Local::now(),
            children: Vec::new(),
        }
    }
    
    pub fn from_todo(todo: &Todo, name: String) -> Self {
        let template_id = format!("template_{}", Local::now().timestamp_millis());
        
        Self {
            id: template_id,
            name,
            description: todo.description.clone(),
            tags: todo.tags.clone(),
            contexts: todo.contexts.clone(),
            priority: todo.priority,
            recurrence: todo.recurrence.clone(),
            notes: todo.notes.clone(),
            created_at: Local::now(),
            children: Vec::new(), // For now, we don't include children in templates
        }
    }
    
    pub fn apply_to_todo(&self, todo: &mut Todo) {
        todo.tags = self.tags.clone();
        todo.contexts = self.contexts.clone();
        todo.priority = self.priority;
        todo.recurrence = self.recurrence.clone();
        todo.notes = self.notes.clone();
        todo.template_id = Some(self.id.clone());
    }
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }
    
    pub fn add_template(&mut self, template: TodoTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
    
    pub fn remove_template(&mut self, id: &str) -> Option<TodoTemplate> {
        self.templates.remove(id)
    }
    
    pub fn get_template(&self, id: &str) -> Option<&TodoTemplate> {
        self.templates.get(id)
    }
    
    pub fn get_all_templates(&self) -> Vec<&TodoTemplate> {
        let mut templates: Vec<&TodoTemplate> = self.templates.values().collect();
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        templates
    }
    
    pub fn create_template_from_todo(&mut self, todo: &Todo, name: String) -> String {
        let template = TodoTemplate::from_todo(todo, name);
        let id = template.id.clone();
        self.add_template(template);
        id
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

// Add some built-in templates
impl TemplateManager {
    pub fn with_builtin_templates() -> Self {
        let mut manager = Self::new();
        
        // Work task template
        let mut work_template = TodoTemplate {
            id: "builtin-work-task".to_string(),
            name: "Work Task".to_string(),
            description: "".to_string(),
            tags: HashSet::new(),
            contexts: HashSet::new(),
            priority: 2,
            recurrence: RecurrencePattern::None,
            notes: None,
            created_at: Local::now(),
            children: Vec::new(),
        };
        work_template.contexts.insert("work".to_string());
        work_template.tags.insert("task".to_string());
        manager.add_template(work_template);
        
        // Personal task template
        let mut personal_template = TodoTemplate {
            id: "builtin-personal-task".to_string(),
            name: "Personal Task".to_string(),
            description: "".to_string(),
            tags: HashSet::new(),
            contexts: HashSet::new(),
            priority: 1,
            recurrence: RecurrencePattern::None,
            notes: None,
            created_at: Local::now(),
            children: Vec::new(),
        };
        personal_template.contexts.insert("personal".to_string());
        personal_template.tags.insert("life".to_string());
        manager.add_template(personal_template);
        
        // Bug report template
        let mut bug_template = TodoTemplate {
            id: "builtin-bug-report".to_string(),
            name: "Bug Report".to_string(),
            description: "".to_string(),
            tags: HashSet::new(),
            contexts: HashSet::new(),
            priority: 4,
            recurrence: RecurrencePattern::None,
            notes: Some("Steps to reproduce:\n1. \n2. \n3. \n\nExpected behavior:\n\nActual behavior:\n\nPossible fix:".to_string()),
            created_at: Local::now(),
            children: Vec::new(),
        };
        bug_template.contexts.insert("development".to_string());
        bug_template.tags.insert("bug".to_string());
        manager.add_template(bug_template);
        
        // Meeting notes template
        let mut meeting_template = TodoTemplate {
            id: "builtin-meeting-notes".to_string(),
            name: "Meeting Notes".to_string(),
            description: "".to_string(),
            tags: HashSet::new(),
            contexts: HashSet::new(),
            priority: 1,
            recurrence: RecurrencePattern::None,
            notes: Some("Agenda:\n- \n- \n- \n\nNotes:\n- \n- \n- \n\nAction items:\n- \n- ".to_string()),
            created_at: Local::now(),
            children: Vec::new(),
        };
        meeting_template.contexts.insert("meetings".to_string());
        meeting_template.tags.insert("meeting".to_string());
        manager.add_template(meeting_template);
        
        manager
    }
}
