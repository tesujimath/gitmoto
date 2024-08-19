use anyhow::Result;

use filesystem::GitDirs;
use globset::GlobSet;
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

pub mod filesystem; // Filesystem traversal

async fn tokio_main() -> Result<()> {
    let mut g = GitDirs::new(["/home/sjg/vc", "/home/sjg/junk"], GlobSet::empty());

    while let Some(dir) = g.next().await {
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
