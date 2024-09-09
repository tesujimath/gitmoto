use anyhow::Result;

use futures::pin_mut;
use globset::GlobSet;
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

async fn tokio_main() -> Result<()> {
    let local = filesystem::git_dirs(["/home/sjg/vc", "/home/sjg/junk"], GlobSet::empty());
    pin_mut!(local);

    let remote = ssh::git_dirs("git", "localhost", ["."], GlobSet::empty());
    pin_mut!(remote);

    // while let Some(dir) = local.next().await {
    //     println!("{}", dir.to_string_lossy());
    // }

    while let Some(dir) = remote.next().await {
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

    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}

pub mod filesystem; // Filesystem traversal
pub mod ssh; // ssh remote traversal
