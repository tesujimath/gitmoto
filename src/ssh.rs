use anyhow::Result;
use futures::{Stream, StreamExt};
use openssh::{KnownHosts, Session};
use std::{
    collections::VecDeque,
    fmt::Debug,
    future::Future,
    mem,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::fs::{read_dir, symlink_metadata};
use tokio_stream::wrappers::ReadDirStream;
use tracing::trace;

/// stream returning all git directories under `rootdir`
pub struct GitDirs {
    destination: String,
    session: Session,
    pending_dirs: VecDeque<PathBuf>,
    state: GitDirsState,
}

// futures we could be awaiting
enum GitDirsState {
    Initial,
    IsGitDir(PathBuf, Pin<Box<dyn Future<Output = bool>>>),
    QueueSubdirs(Pin<Box<dyn Future<Output = Vec<PathBuf>>>>),
}

impl GitDirs {
    pub async fn connect<S, I, P>(destination: S, rootdirs: I) -> Result<Self>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let destination = destination.as_ref();
        let session = Session::connect_mux(destination, KnownHosts::Add).await?;
        let this = Self {
            destination: destination.to_string(),
            session,
            pending_dirs: VecDeque::from_iter(
                rootdirs.into_iter().map(|p| p.as_ref().to_path_buf()),
            ),
            state: GitDirsState::Initial,
        };
        trace!(
            "GitDirs::new destination={} pending_dirs={:?}",
            this.destination,
            this.pending_dirs
        );
        Ok(this)
    }
}

#[tracing::instrument(level = "trace")]
async fn read_subdirs<P>(dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path> + Debug,
{
    let dir = dir.as_ref();
    if let Ok(rd) = read_dir(dir).await {
        let rds = ReadDirStream::new(rd);
        rds.filter_map(|entry| async move {
            match entry {
                Ok(entry) => {
                    let entry_path = dir.join(entry.path());
                    is_dir(&entry_path).await.then_some(entry_path)
                }
                Err(_) => None,
            }
        })
        .collect::<Vec<PathBuf>>()
        .await
    } else {
        Vec::default()
    }
}
