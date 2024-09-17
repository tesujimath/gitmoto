use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs::OpenOptions, io, path::PathBuf};
use tokio::select;
use tracing::trace;
use tracing_subscriber::EnvFilter;

use crate::{
    app::App,
    handler::{handle_filesystem_event, handle_key_events},
    render::render,
    service::{filesystem, terminal},
    tui::Tui,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let logfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("{}.log", env!("CARGO_PKG_NAME")))
        .unwrap();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(logfile)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    trace!("");
    trace!("                    STARTING");
    trace!("");

    // Create an application.
    let mut app = App::default();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);
    tui.init()?;

    let mut terminal_service = terminal::Service::default();
    let mut filesystem_service = filesystem::Service::default();
    let filesystem_requester = filesystem_service.requester();

    filesystem_requester
        .send(filesystem::Request::Scan(PathBuf::from("/home/sjg/vc")))
        .await
        .unwrap();

    // Start the main loop.
    // let mut terminal_closed = false;
    while app.running {
        // Render the user interface.
        tui.draw(|frame| render(&app.presenter, frame))?;
        // Handle events.
        select! {
            terminal_event = terminal_service.recv_event()/*, if !terminal_closed*/ => {
                match terminal_event {
                    None => {
                        /*terminal_closed = true;*/
                        trace!("None from terminal_service::recv_event");
                         }
                    Some(terminal::Event::Key(key_event)) => handle_key_events(key_event, &mut app),
                    Some(terminal::Event::Mouse(_)) => {}
                    Some(terminal::Event::Resize(_, _)) => {}
                }
            }
            filesystem_event = filesystem_service.recv_event() => {
                if let Some(filesystem_event) = filesystem_event {
                handle_filesystem_event(filesystem_event, &mut app.presenter.model);
                }
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}

pub mod app;
pub mod github; // GitHub API
pub mod handler;
pub mod model;
pub mod presenter;
pub mod render;
pub mod service;
pub mod ssh; // ssh remote traversal
pub mod tui;
