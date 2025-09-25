use crate::app::{App, AppMode};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub fn handle_event(app: &mut App, event: Event) -> io::Result<()> {
    // Clear message on any key press
    if matches!(event, Event::Key(_)) {
        app.clear_message();
    }

    match event {
        Event::Key(key_event) => handle_key_event(app, key_event),
        _ => Ok(()),
    }
}

fn handle_key_event(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    // Help screen - any key closes it
    if app.show_help {
        app.toggle_help();
        return Ok(());
    }

    match app.mode {
        AppMode::Welcome => handle_welcome_mode(app, key_event)?,
        AppMode::Normal => handle_normal_mode(app, key_event)?,
        AppMode::Insert | AppMode::InsertChild => handle_insert_mode(app, key_event)?,
        AppMode::EditTodo => handle_edit_mode(app, key_event)?,
        AppMode::Search => handle_search_mode(app, key_event)?,
        AppMode::TagSelection | AppMode::ContextSelection | AppMode::TemplateSelection | AppMode::RecurrenceSelection | AppMode::WorkspaceSelection => handle_popup_mode(app, key_event)?,
        AppMode::EditNotes => handle_notes_mode(app, key_event)?,
        AppMode::ViewNotes => handle_view_notes_mode(app, key_event)?,
        AppMode::TimeTracking => handle_normal_mode(app, key_event)?, // For now, same as normal
        AppMode::CreateWorkspace => handle_create_workspace_mode(app, key_event)?,
        AppMode::Visual => handle_visual_mode(app, key_event)?,
        AppMode::BulkOperation => handle_bulk_operation_mode(app, key_event)?,
    }
    
    Ok(())
}

fn handle_normal_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Quit
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.quit();
        }
        
        // Clear filters/escape
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.clear_filters();
        }

        // Help
        KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.toggle_help();
        }

        // Navigation
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.move_selection_down();
        }

        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.move_selection_up();
        }

        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.go_to_top();
        }

        KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.go_to_bottom();
        }

        // Actions
        KeyEvent {
            code: KeyCode::Char('i'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_insert_mode();
        }

        KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.toggle_todo_complete();
        }

        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.delete_selected_todo();
        }

        KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.cycle_view_mode();
        }
        
        // View notes (read-only)
        KeyEvent {
            code: KeyCode::Char('V'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.enter_view_notes_mode();
        }

        // Priority
        KeyEvent {
            code: KeyCode::Char('+'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('='),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.increase_priority();
        }

        KeyEvent {
            code: KeyCode::Char('-'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.decrease_priority();
        }

        // Hierarchical operations
        KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.add_child_todo();
        }

        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.toggle_expansion();
        }

        KeyEvent {
            code: KeyCode::Char('D'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.delete_selected_with_children();
        }

        // Search and filtering
        KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_search_mode();
        }

        KeyEvent {
            code: KeyCode::Char('#'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_tag_selection();
        }

        KeyEvent {
            code: KeyCode::Char('@'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_context_selection();
        }

        KeyEvent {
            code: KeyCode::Char('!'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.cycle_due_date_filter();
        }

        // Advanced features
        KeyEvent {
            code: KeyCode::Char('t'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.toggle_timer();
        }

        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_notes_mode();
        }
        
        // Edit todo description
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_edit_mode();
        }

        KeyEvent {
            code: KeyCode::Char('T'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.enter_template_selection();
        }

        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_recurrence_selection();
        }
        
        // Workspace selection
        KeyEvent {
            code: KeyCode::Char('w'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.enter_workspace_selection();
        }
        
        // Return to welcome screen
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.return_to_welcome();
        }
        
        // Undo
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.undo();
        }
        
        // Redo
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.redo();
        }
        
        // Visual mode (bulk operations)
        KeyEvent {
            code: KeyCode::Char('V'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.enter_visual_mode();
        }

        _ => {}
    }

    Ok(())
}

fn handle_insert_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Submit
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.submit_input();
        }

        // Cancel
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.enter_normal_mode();
        }

        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            app.remove_char_from_input();
        }

        // Cursor navigation - left arrow
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => {
            app.move_input_cursor_left();
        }
        
        // Cursor navigation - right arrow
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => {
            app.move_input_cursor_right();
        }

        // Character input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.add_char_to_input(c);
        }

        _ => {}
    }

    Ok(())
}

fn handle_popup_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Select item
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.select_from_popup();
        }

        // Cancel popup
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.cancel_popup();
        }

        // Navigation
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.move_popup_selection_down();
        }

        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.move_popup_selection_up();
        }
        
        // Workspace-specific actions
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Only allow creating new workspace from workspace selection mode
            if app.mode == AppMode::WorkspaceSelection {
                app.enter_create_workspace_mode();
            }
        }
        
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Only allow deleting workspace from workspace selection mode
            if app.mode == AppMode::WorkspaceSelection {
                app.delete_selected_workspace();
            }
        }

        _ => {}
    }

    Ok(())
}

fn handle_search_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Submit search
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.submit_search();
        }

        // Cancel search
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.enter_normal_mode();
        }

        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            app.remove_char_from_search();
        }

        // Cursor navigation - left arrow
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => {
            app.move_search_cursor_left();
        }
        
        // Cursor navigation - right arrow
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => {
            app.move_search_cursor_right();
        }

        // Character input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.add_char_to_search(c);
        }

        _ => {}
    }

    Ok(())
}

fn handle_edit_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Save edit
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.save_todo_edit();
        }

        // Cancel editing
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.exit_edit_mode();
        }

        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            app.remove_char_from_edit();
        }

        // Cursor navigation - left arrow
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => {
            app.move_edit_cursor_left();
        }
        
        // Cursor navigation - right arrow
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => {
            app.move_edit_cursor_right();
        }

        // Character input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.add_char_to_edit(c);
        }

        _ => {}
    }

    Ok(())
}

fn handle_notes_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Save notes - Ctrl+Enter
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.save_notes();
        }
        
        // Alternative save - Ctrl+S
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => {
            app.save_notes();
        }
        
        // Simple save - F2 key (no modifiers needed)
        KeyEvent {
            code: KeyCode::F(2),
            ..
        } => {
            app.save_notes();
        }

        // Cancel notes editing
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.exit_notes_mode();
        }

        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            app.remove_char_from_notes();
        }

        // Enter creates new line
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.add_char_to_notes('\n');
        }

        // Cursor navigation - left arrow
        KeyEvent {
            code: KeyCode::Left,
            ..
        } => {
            app.move_notes_cursor_left();
        }
        
        // Cursor navigation - right arrow
        KeyEvent {
            code: KeyCode::Right,
            ..
        } => {
            app.move_notes_cursor_right();
        }

        // Character input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.add_char_to_notes(c);
        }

        _ => {}
    }

    Ok(())
}

fn handle_view_notes_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Close notes viewer
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.exit_view_notes_mode();
        }
        
        // Switch to edit mode
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Switch from view to edit mode
            app.mode = AppMode::EditNotes;
        }

        _ => {}
    }

    Ok(())
}

fn handle_visual_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Exit visual mode
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.exit_visual_mode();
        }
        
        // Navigation in visual mode
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.move_selection_down();
            app.select_range_in_visual();
        }

        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.move_selection_up();
            app.select_range_in_visual();
        }
        
        // Toggle individual selection
        KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.toggle_selection_in_visual();
        }
        
        // Bulk operations
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.bulk_complete_todos();
        }
        
        KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.bulk_delete_todos();
        }
        
        // Priority setting (1-5)
        KeyEvent {
            code: KeyCode::Char(c @ '1'..='5'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            let priority = (c as u8) - b'0';
            app.bulk_set_priority(priority);
        }
        
        KeyEvent {
            code: KeyCode::Char('0'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.bulk_set_priority(0);
        }
        
        _ => {}
    }

    Ok(())
}

fn handle_bulk_operation_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Exit bulk operation mode
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.exit_visual_mode();
        }
        
        _ => {}
    }

    Ok(())
}

fn handle_welcome_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Navigation - move up
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            ..
        } => {
            app.move_welcome_selection_up();
        }

        // Navigation - move down
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            ..
        } => {
            app.move_welcome_selection_down();
        }

        // Select option
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.select_welcome_option();
        }

        // Show help
        KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            app.toggle_help();
        }

        // Quick shortcuts for common actions
        KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Quick shortcut for "Get Started"
            app.welcome_selected = 0;
            app.select_welcome_option();
        }

        KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Quick shortcut for "Browse Workspaces"
            app.welcome_selected = 1;
            app.select_welcome_option();
        }

        KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Quick shortcut for "Learn the Basics"
            app.welcome_selected = 2;
            app.select_welcome_option();
        }

        KeyEvent {
            code: KeyCode::Char('4'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            // Quick shortcut for "Quick Demo"
            app.welcome_selected = 3;
            app.select_welcome_option();
        }

        // Quit
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
        | KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.quit();
        }

        _ => {}
    }

    Ok(())
}

fn handle_create_workspace_mode(app: &mut App, key_event: KeyEvent) -> io::Result<()> {
    match key_event {
        // Submit workspace creation
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            app.submit_workspace_creation();
        }

        // Cancel workspace creation
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } => {
            app.cancel_workspace_creation();
        }

        // Backspace
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            app.remove_char_from_input();
        }

        // Character input
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => {
            app.add_char_to_input(c);
        }

        _ => {}
    }

    Ok(())
}
