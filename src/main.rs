mod app;
mod colors;
mod events;
mod storage;
mod template;
mod todo;
mod ui;
mod tests;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::stdout,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and load data
    let mut app = App::new();
    let storage = storage::Storage::new()?;
    
    // Load workspace manager (this will handle migration from legacy format)
    match storage.load_workspace_manager() {
        Ok(workspace_manager) => {
            app.workspace_manager = workspace_manager;
            
            // Refresh available workspaces for selection
            app.available_workspaces = app.workspace_manager.get_all_workspaces()
                .iter()
                .map(|ws| ws.name.clone())
                .collect();
            
            // Count total todos across all workspaces
            let total_todos: usize = app.workspace_manager.workspace_todos.values()
                .map(|todo_list| todo_list.total_count())
                .sum();
            if total_todos > 0 {
                app.set_message(format!("Loaded {} todos across {} workspaces. Select a workspace to continue.", 
                    total_todos, app.workspace_manager.workspaces.len()));
            } else {
                app.set_message("Select a workspace to get started".to_string());
            }
        }
        Err(e) => {
            app.set_message(format!("Failed to load workspaces: {}", e));
        }
    }

    // Main loop
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    
    let result = loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let Err(e) = events::handle_event(&mut app, Event::Key(key)) {
                    break Err(e.into());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            break Ok(());
        }
    };

    // Save workspace manager before exiting
    if let Err(e) = storage.save_workspace_manager(&app.workspace_manager) {
        eprintln!("Failed to save workspace data: {}", e);
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
