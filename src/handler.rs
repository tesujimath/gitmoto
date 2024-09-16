use crate::filesystem;
use crate::{app::App, model::Model};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key_event.code {
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit();
        }

        // Other handlers you could add here.
        _ => {
            app.key(key_event);
        }
    }
    Ok(())
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_filesystem_event(filesystem_event: filesystem::Event, model: &mut Model) {
    use filesystem::Event::*;

    match filesystem_event {
        LocalRepo(path) => {
            model.add_local_repo(path);
        }
    }
}
