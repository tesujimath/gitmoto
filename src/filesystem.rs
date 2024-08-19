use futures::{Stream, StreamExt};
use globset::GlobSet;
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
    _excludes: GlobSet,
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
    pub fn new<I, P>(rootdirs: I, excludes: GlobSet) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let this = Self {
            _excludes: excludes,
            pending_dirs: VecDeque::from_iter(
                rootdirs.into_iter().map(|p| p.as_ref().to_path_buf()),
            ),
            state: GitDirsState::Initial,
        };
        trace!("GitDirs::new pending_dirs={:?}", this.pending_dirs);
        this
    }
}

#[tracing::instrument(level = "trace")]
async fn read_subdirs<P>(dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path> + Debug,
{
    let dir = dir.as_ref();
    // println!("queue_subdirs {:?}", dir);
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

#[tracing::instrument(level = "trace")]
async fn is_dir<P>(path: P) -> bool
where
    P: AsRef<Path> + Debug,
{
    let path = path.as_ref();
    symlink_metadata(path).await.is_ok_and(|m| m.is_dir())
}

impl Stream for GitDirs {
    type Item = PathBuf;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = GitDirsState::Initial;
        mem::swap(&mut self.state, &mut state);
        match state {
            GitDirsState::Initial => match self.pending_dirs.pop_front() {
                None => {
                    trace!("GitDirs::Stream done");
                    Poll::Ready(None)
                }
                Some(dir) => {
                    let gitdir_candidate = dir.join(".git");
                    let s = GitDirsState::IsGitDir(dir, Box::pin(is_dir(gitdir_candidate)));
                    self.state = s;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            GitDirsState::IsGitDir(path, mut is_git_dir) => match is_git_dir.as_mut().poll(cx) {
                Poll::Ready(is_dir) => {
                    if is_dir {
                        trace!("GitDirs::Stream yield {:?}", &path);
                        Poll::Ready(Some(path))
                    } else {
                        trace!("GitDirs::Stream queue {:?}", path);
                        let q = Box::pin(read_subdirs(path));
                        let mut state = GitDirsState::QueueSubdirs(q);
                        mem::swap(&mut self.state, &mut state);
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
                Poll::Pending => {
                    let mut state = GitDirsState::IsGitDir(path, is_git_dir);
                    mem::swap(&mut self.state, &mut state);
                    Poll::Pending
                }
            },
            GitDirsState::QueueSubdirs(mut queue_subdirs) => {
                match queue_subdirs.as_mut().poll(cx) {
                    Poll::Ready(subdirs) => {
                        self.pending_dirs.extend(subdirs);
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Poll::Pending => {
                        let mut state = GitDirsState::QueueSubdirs(queue_subdirs);
                        mem::swap(&mut self.state, &mut state);
                        Poll::Pending
                    }
                }
            }
        }
    }
}
