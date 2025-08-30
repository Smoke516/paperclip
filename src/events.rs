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
        AppMode::Normal => handle_normal_mode(app, key_event),
        AppMode::Insert => handle_insert_mode(app, key_event),
        AppMode::InsertChild => handle_insert_mode(app, key_event), // Same as insert mode
        AppMode::EditTodo => handle_edit_mode(app, key_event),
        AppMode::Search => handle_search_mode(app, key_event),
        AppMode::TagSelection | AppMode::ContextSelection | AppMode::TemplateSelection | AppMode::RecurrenceSelection | AppMode::WorkspaceSelection => handle_popup_mode(app, key_event),
        AppMode::EditNotes => handle_notes_mode(app, key_event),
        AppMode::ViewNotes => handle_view_notes_mode(app, key_event),
        AppMode::TimeTracking => handle_normal_mode(app, key_event), // For now, same as normal
        AppMode::CreateWorkspace => handle_create_workspace_mode(app, key_event),
    }
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
