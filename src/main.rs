use std::{fs::OpenOptions, io, path::PathBuf};

use anyhow::Result;
use app::App;
use futures::pin_mut;
use globset::GlobSet;
use handler::{handle_filesystem_event, handle_key_events};
use ratatui::{backend::CrosstermBackend, Terminal};
use terminal_event::{EventHandler, TerminalEvent};
use tokio::{select, sync::mpsc};
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;
use tui::Tui;

async fn tokio_main() -> Result<()> {
    // let local = filesystem::git_dirs(["/home/sjg/vc", "/home/sjg/junk"], GlobSet::empty());
    // pin_mut!(local);

    // let remote = ssh::git_dirs("git", "localhost", ["."], GlobSet::empty());
    // pin_mut!(remote);

    // let github = github::Connection::new();
    // let github_repos = github.git_dirs("tesujimath");
    // pin_mut!(github_repos);

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

    // while let Some(dir) = github_repos.next().await {
    //     println!("{}", dir);
    // }

    Ok(())
}

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

    // Create an application.
    let mut app = App::default();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    let (warning_tx, warning_rx) = mpsc::channel(1);

    let (filesystem_event_tx, mut filesystem_event_rx) = mpsc::channel(1);
    let (filesystem_request_tx, filesystem_request_rx) = mpsc::channel(1);
    tokio::spawn(filesystem::worker(
        filesystem_request_rx,
        filesystem_event_tx,
        warning_tx,
    ));

    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // initiate a filesystem scan
    filesystem_request_tx
        .send(filesystem::Request::Scan(PathBuf::from("/home/sjg/vc")))
        .await
        .unwrap();

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        select! {
            tui_event = tui.events.next() => {
                match tui_event? {
                    TerminalEvent::Tick => app.tick(),
                    TerminalEvent::Key(key_event) => handle_key_events(key_event, &mut app)?,
                    TerminalEvent::Mouse(_) => {}
                    TerminalEvent::Resize(_, _) => {}
                }
            }
            filesystem_event = filesystem_event_rx.recv() => {
                if let Some(filesystem_event) = filesystem_event {
                handle_filesystem_event(filesystem_event, &mut app);
                }
            }
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

pub mod app;
pub mod filesystem; // Filesystem traversal
pub mod github; // GitHub API
pub mod handler;
pub mod ssh; // ssh remote traversal
pub mod terminal_event;
pub mod tui;
pub mod ui;
