use std::io;

use anyhow::Result;
use futures::pin_mut;
use globset::GlobSet;
use handler::handle_key_events;
use model::Model;
use ratatui::{backend::CrosstermBackend, Terminal};
use terminal_event::{EventHandler, TerminalEvent};
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;
use tui::Tui;

async fn tokio_main() -> Result<()> {
    let local = filesystem::git_dirs(["/home/sjg/vc", "/home/sjg/junk"], GlobSet::empty());
    pin_mut!(local);

    let remote = ssh::git_dirs("git", "localhost", ["."], GlobSet::empty());
    pin_mut!(remote);

    let github = github::Connection::new();
    let github_repos = github.git_dirs("tesujimath");
    pin_mut!(github_repos);

    // while let Some(dir) = local.next().await {
    //     match dir {
    //         Ok(dir) => {
    //             println!("{}", dir.to_string_lossy());
    //         }
    //         Err(e) => {
    //             eprintln!("{:?}", e);
    //         }
    //     }
    // }

    // while let Some(dir) = remote.next().await {
    //     println!("{}", dir);
    // }

    while let Some(dir) = github_repos.next().await {
        println!("{}", dir);
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create an application.
    let mut model = Model::default();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    // let (worker_out_tx, worker_out_rx) = channel(1);
    // tokio::spawn(worker(worker_out_tx));
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while model.running {
        // Render the user interface.
        tui.draw(&mut model)?;
        // Handle events.
        match tui.events.next().await? {
            TerminalEvent::Tick => model.tick(),
            TerminalEvent::Key(key_event) => handle_key_events(key_event, &mut model)?,
            TerminalEvent::Mouse(_) => {}
            TerminalEvent::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    //     if let Err(e) = tokio_main().await {
    //     eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    //     Err(e)
    // } else {
    //     Ok(())
    // }
    Ok(())
}

pub mod filesystem; // Filesystem traversal
pub mod github; // GitHub API
pub mod handler;
pub mod model;
pub mod ssh; // ssh remote traversal
pub mod terminal_event;
pub mod tui;
pub mod ui;
