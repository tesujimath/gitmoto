use async_stream::stream;
use futures::Stream;
use globset::GlobSet;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};
use tokio::{
    fs::read_dir,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;

/// stream returning all git directories under `rootdir`
#[derive(Debug)]
pub struct GitDirs {
    excludes: GlobSet,
    pending_dirs: VecDeque<PathBuf>,
}

impl GitDirs {
    pub fn new<I, P>(rootdirs: I, excludes: GlobSet) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        Self {
            excludes,
            pending_dirs: VecDeque::from_iter(
                rootdirs.into_iter().map(|p| p.as_ref().to_path_buf()),
            ),
        }
    }

    async fn queue_subdirs<P>(&mut self, dir: P)
    where
        P: AsRef<Path>,
    {
        let dir = dir.as_ref();
        if let Ok(rd) = read_dir(dir).await {
            let mut rds = tokio_stream::wrappers::ReadDirStream::new(rd);
            while let Some(entry) = rds.next().await {
                // let paths = rd.filter_map(|entry| {
                let path = entry.ok().and_then(|entry| {
                    let entry_path = dir.join(entry.path());
                    (!entry_path.is_symlink() && entry_path.is_dir()).then_some(entry_path)
                });

                if let Some(path) = path {
                    self.pending_dirs.push_back(path);
                }
            }
        }
    }
}

impl Stream for GitDirs {
    type Item = PathBuf;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.pending_dirs.pop_front())
    }
}

pub fn git_dirs<I, P>(_rootdirs: I, _excludes: GlobSet) -> impl Stream<Item = PathBuf> {
    stream! {
        yield PathBuf::from("/home/sjg/vc/sjg/dev.rust/async-playpen");
        yield PathBuf::from("/home/sjg/vc/sjg/dev.rust/playpen");
    }
    // WalkDir::new(rootdir)
    //     .into_iter()
    //     .filter_entry(|e| e.file_type().is_dir() && !excludes.is_match(e.path()))
    //     .filter_map(|e| match e {
    //         Ok(e) if e.file_name() == ".git" => Some(e.into_path()),
    //         _ => None,
    //     })
}
