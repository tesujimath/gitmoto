use std::io::Result;

use filesystem::GitDirs;
use globset::GlobSet;
use tokio_stream::StreamExt;

pub mod filesystem; // Filesystem traversal

async fn tokio_main() -> Result<()> {
    let mut g = GitDirs::new(
        ["/home/sjg/vc/sjg/dev.rust/playpen", "/home/sjg/junk"],
        GlobSet::empty(),
    );

    while let Some(dir) = g.next().await {
        println!("found dir {:?}", dir);
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
