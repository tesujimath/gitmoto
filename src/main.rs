use anyhow::Result;
use config::read_config;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{fs::OpenOptions, io, process::exit};
use tokio::select;
use tracing::trace;
use tracing_subscriber::EnvFilter;

use crate::{
    model::UpdateModel,
    presenter::Presenter,
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

    let config = read_config().expect("Failed to read config");

    // Create an application.
    let mut presenter = Presenter::default();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);
    tui.init()?;

    let mut terminal_service = terminal::Service::default();
    let mut filesystem_service = filesystem::Service::new(&config.filesystem);
    let filesystem_requester = filesystem_service.requester();

    let roots = &config.filesystem.scanner.roots;
    if roots.is_empty() {
        eprintln!("no filesystem scanner roots found in config, abort");
        exit(1);
    }

    // initial filesystem scan, TODO read from cache
    filesystem_requester
        .send(filesystem::Request::Scan)
        .await
        .unwrap();

    // Start the main loop.
    let mut running = true;
    while running {
        // Render the user interface.
        tui.draw(|frame| presenter.render(frame, &config.display))?;
        // Handle events.
        select! {
            ev = terminal_service.recv_event()  => {
                if let Some(ev) = ev {
                    let quit = terminal_service.handle(ev, |key| presenter.handle_key(key)).await;
                    if quit {
                        running = false;
                    }
                }
            },
            key = filesystem_service.recv_event() => {
                if let Some(key) = key {
                    filesystem_service.handle(key, |repo| presenter.add_local_repo(repo)).await;
                }
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;

    Ok(())
}

pub mod config;
pub mod github; // GitHub API
pub mod model;
pub mod presenter;
pub mod service;
pub mod ssh; // ssh remote traversal
pub mod template;
pub mod tui;
pub mod util;
