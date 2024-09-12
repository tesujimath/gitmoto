use anyhow::{Context, Result};
use async_stream::stream;
use futures::{Stream, StreamExt};
use globset::GlobSet;
use std::{
    collections::VecDeque,
    fmt::Debug,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{read_dir, symlink_metadata},
    sync::mpsc,
};
use tokio_stream::wrappers::ReadDirStream;
use tracing::trace;

pub enum Request {
    Scan(PathBuf),
}

pub enum Event {
    GitDir(PathBuf),
}

pub async fn worker(
    mut request_rx: mpsc::Receiver<Request>,
    event_tx: mpsc::Sender<Event>,
    warning_tx: mpsc::Sender<String>,
) {
    while let Some(request) = request_rx.recv().await {
        use Request::*;

        match request {
            Scan(rootdir) => {
                trace!("filesystem::Scan({:?})", &rootdir);
                let mut pending_dirs = VecDeque::from([rootdir]);

                while let Some(dir) = pending_dirs.pop_front() {
                    let gitdir_candidate = dir.join(".git");

                    if is_dir(gitdir_candidate).await {
                        trace!("filesystem::Scan sending {:?}", &dir);
                        event_tx.send(Event::GitDir(dir)).await.unwrap();
                        trace!("filesystem::Scan sent")
                    } else {
                        match read_subdirs(&dir).await {
                            Ok(subdirs) => {
                                pending_dirs.extend(subdirs);
                            }
                            Err(e) => {
                                let warning = e.to_string();
                                trace!("filesystem::Scan warning {}", &warning);
                                warning_tx.send(warning).await.unwrap();
                                trace!("filesystem::Scan warned")
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn git_dirs<I, P>(rootdirs: I, _excludes: GlobSet) -> impl Stream<Item = Result<PathBuf>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut pending_dirs =
        VecDeque::from_iter(rootdirs.into_iter().map(|p| p.as_ref().to_path_buf()));
    trace!("git_dirs(pending_dirs={:?})", pending_dirs);

    stream! {
        while let Some(dir)  = pending_dirs.pop_front() {
            let gitdir_candidate = dir.join(".git");

            if is_dir(gitdir_candidate).await {
                yield Ok(dir);
            } else {
                match read_subdirs(&dir).await {
                    Ok(subdirs) => {
                        pending_dirs.extend(subdirs);
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }

        }
    }
}

#[tracing::instrument(level = "trace")]
async fn read_subdirs<P>(dir: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path> + Debug,
{
    let dir = dir.as_ref();
    let rd = read_dir(dir)
        .await
        .with_context(|| format!("Failed to read {}", dir.to_string_lossy()))?;
    let rds = ReadDirStream::new(rd);
    Ok(rds
        .filter_map(|entry| async move {
            match entry {
                Ok(entry) => {
                    let entry_path = dir.join(entry.path());
                    is_dir(&entry_path).await.then_some(entry_path)
                }
                Err(_) => None,
            }
        })
        .collect::<Vec<PathBuf>>()
        .await)
}

#[tracing::instrument(level = "trace")]
async fn is_dir<P>(path: P) -> bool
where
    P: AsRef<Path> + Debug,
{
    let path = path.as_ref();
    symlink_metadata(path).await.is_ok_and(|m| m.is_dir())
}
