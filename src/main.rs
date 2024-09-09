use anyhow::Result;

use futures::pin_mut;
use globset::GlobSet;
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

async fn tokio_main() -> Result<()> {
    let mut local = filesystem::git_dirs(["/home/sjg/vc", "/home/sjg/junk"], GlobSet::empty());
    let mut remote = ssh::GitDirs::connect("git", "localhost", ["."]).await?;
    pin_mut!(local);

    while let Some(dir) = local.next().await {
        println!("{}", dir.to_string_lossy());
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}

pub mod filesystem; // Filesystem traversal
pub mod ssh; // ssh remote traversal
