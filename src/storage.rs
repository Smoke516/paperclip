use crate::todo::{TodoList, WorkspaceManager};
use serde_json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Storage {
    data_file: PathBuf,
    workspace_file: PathBuf,
}

impl Storage {
    pub fn new() -> io::Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find data directory"))?
            .join("paperclip");
        
        // Create data directory if it doesn't exist
        fs::create_dir_all(&data_dir)?;
        
        let data_file = data_dir.join("todos.json");
        let workspace_file = data_dir.join("workspaces.json");
        
        Ok(Self { data_file, workspace_file })
    }

    // Legacy method for backward compatibility
    pub fn load_todos(&self) -> io::Result<TodoList> {
        if !self.data_file.exists() {
            return Ok(TodoList::new());
        }

        let content = fs::read_to_string(&self.data_file)?;
        let todo_list: TodoList = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(todo_list)
    }

    // Legacy method for backward compatibility
    pub fn save_todos(&self, todo_list: &TodoList) -> io::Result<()> {
        let content = serde_json::to_string_pretty(todo_list)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(&self.data_file, content)?;
        Ok(())
    }
    
    // New workspace-based methods
    pub fn load_workspace_manager(&self) -> io::Result<WorkspaceManager> {
        if !self.workspace_file.exists() {
            // If no workspace file exists, try to migrate from old format
            return self.migrate_from_legacy();
        }

        let content = fs::read_to_string(&self.workspace_file)?;
        let workspace_manager: WorkspaceManager = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(workspace_manager)
    }

    pub fn save_workspace_manager(&self, workspace_manager: &WorkspaceManager) -> io::Result<()> {
        let content = serde_json::to_string_pretty(workspace_manager)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        fs::write(&self.workspace_file, content)?;
        Ok(())
    }
    
    // Migration from legacy single TodoList to WorkspaceManager
    fn migrate_from_legacy(&self) -> io::Result<WorkspaceManager> {
        let mut workspace_manager = WorkspaceManager::new();
        
        // Create a default workspace
        let workspace_id = workspace_manager.create_workspace(
            "Personal".to_string(), 
            Some("Migrated from legacy todos".to_string())
        );
        
        // If there's a legacy todos.json file, load it into the default workspace
        if self.data_file.exists() {
            if let Ok(legacy_todos) = self.load_todos() {
                if let Some(todo_list) = workspace_manager.workspace_todos.get_mut(&workspace_id) {
                    *todo_list = legacy_todos;
                }
            }
        }
        
        Ok(workspace_manager)
    }

    pub fn get_data_file_path(&self) -> &Path {
        &self.data_file
    }
    
    pub fn get_workspace_file_path(&self) -> &Path {
        &self.workspace_file
    }
}
