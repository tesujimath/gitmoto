use crate::filesystem;
use crate::model::Model;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`Model`].
pub fn handle_key_events(key_event: KeyEvent, model: &mut Model) -> anyhow::Result<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            model.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                model.quit();
            }
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

/// Handles the key events and updates the state of [`Model`].
pub fn handle_filesystem_event(
    filesystem_event: filesystem::Event,
    model: &mut Model,
) -> anyhow::Result<()> {
    use filesystem::Event::*;

    match filesystem_event {
        GitDir(path) => {
            model.add_local_repo(path);
        }
    }

    Ok(())
}
