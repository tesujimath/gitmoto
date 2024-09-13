use crate::app::App;
use crate::filesystem;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_filesystem_event(
    filesystem_event: filesystem::Event,
    app: &mut App,
) -> anyhow::Result<()> {
    use filesystem::Event::*;

    match filesystem_event {
        GitDir(path) => {
            app.add_local_repo(path);
        }
    }

    Ok(())
}
