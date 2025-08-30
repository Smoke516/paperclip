#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::{Todo, TodoList, TodoStatus, RecurrencePattern};
    use crate::template::TemplateManager;
    use chrono::Local;

    #[test]
    fn test_todo_creation_with_advanced_features() {
        let mut todo = Todo::new(1, "Test todo #urgent @work due:today".to_string());
        
        // Test notes
        todo.set_notes(Some("These are test notes".to_string()));
        assert_eq!(todo.notes, Some("These are test notes".to_string()));
        
        // Test time tracking
        todo.start_timer();
        assert!(todo.is_timer_running());
        
        todo.stop_timer();
        assert!(!todo.is_timer_running());
        
        // Test recurrence
        todo.set_recurrence(RecurrencePattern::Daily);
        assert!(todo.is_recurring());
        
        println!("✅ Todo advanced features work correctly");
    }
    
    #[test]
    fn test_todo_list_operations() {
        let mut todo_list = TodoList::new();
        
        // Add todos
        let id1 = todo_list.add_todo("Parent todo #work".to_string());
        let _id2 = todo_list.add_child_todo(id1, "Child todo @development".to_string());
        
        assert_eq!(todo_list.total_count(), 2);
        assert!(todo_list.has_children(id1));
        
        // Test timer operations
        todo_list.start_timer(id1);
        let active_timers = todo_list.get_active_timers();
        assert_eq!(active_timers.len(), 1);
        
        println!("✅ TodoList operations work correctly");
    }
    
    #[test]
    fn test_template_manager() {
        let template_manager = TemplateManager::with_builtin_templates();
        let templates = template_manager.get_all_templates();
        
        // Should have built-in templates
        assert!(!templates.is_empty());
        
        // Debug: print all template names
        let template_names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        println!("Available templates: {:?}", template_names);
        
        // Check for templates that actually exist
        assert!(template_names.contains(&"Work Task"));
        assert!(template_names.contains(&"Personal Task"));
        assert!(template_names.contains(&"Bug Report"));
        assert!(template_names.contains(&"Meeting Notes"));
        
        println!("✅ Template system works correctly");
    }
    
    #[test]
    fn test_serialization() {
        use serde_json;
        
        let mut todo = Todo::new(1, "Test serialization #test @dev".to_string());
        todo.set_notes(Some("Test notes\\nMultiple lines".to_string()));
        todo.start_timer();
        todo.stop_timer();
        todo.set_recurrence(RecurrencePattern::Weekly);
        
        // Test serialization
        let json = serde_json::to_string(&todo).expect("Failed to serialize todo");
        let deserialized: Todo = serde_json::from_str(&json).expect("Failed to deserialize todo");
        
        assert_eq!(todo.id, deserialized.id);
        assert_eq!(todo.description, deserialized.description);
        assert_eq!(todo.notes, deserialized.notes);
        assert_eq!(todo.recurrence, deserialized.recurrence);
        
        println!("✅ Serialization works correctly");
    }
}
