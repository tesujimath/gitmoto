use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use russh::{
    client::{self, Session},
    keys::{agent::client::AgentClient, key},
    ChannelId,
};
use russh_sftp::client::{
    fs::{Metadata, ReadDir},
    rawsession::SftpResult,
    SftpSession,
};
use std::{
    collections::VecDeque,
    fmt::Debug,
    future::Future,
    mem,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::fs::{read_dir, symlink_metadata};
use tokio_stream::wrappers::ReadDirStream;
use tracing::{debug, info, trace, warn};

/// stream returning all git directories under `rootdir`
pub struct GitDirs {
    user: String,
    host: String,
    pending_dirs: VecDeque<String>,
    session: client::Handle<Client>,
    sftp_session: SftpSession,
    state: GitDirsState,
}

impl Debug for GitDirs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "GitDirs user={} host={} pending_dirs={:?}",
            &self.user, &self.host, &self.pending_dirs
        ))
    }
}
// futures we could be awaiting
enum GitDirsState {
    Initial,
    IsGitDir(String, Pin<Box<dyn Future<Output = SftpResult<Metadata>>>>),
    ReadSubdirs(Pin<Box<dyn Future<Output = SftpResult<ReadDir>>>>),
}

impl GitDirs {
    pub async fn connect<S1, S2, I, P>(user: S1, host: S2, rootdirs: I) -> Result<Self>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let user = user.as_ref();
        let host = host.as_ref();

        let mut agent = AgentClient::connect_env().await?;
        let identities = agent.request_identities().await?;

        let config = russh::client::Config::default();
        let sh = Client {};
        let mut session = russh::client::connect(Arc::new(config), (host, 22), sh).await?;
        let mut authenticated = false;

        // use the first identity for which authentication succeeds
        for id in identities {
            debug!("trying identity {} {}", id.name(), id.fingerprint());

            let (agent_, authenticated_) = session.authenticate_future(user, id, agent).await;
            agent = agent_;
            authenticated = authenticated_?;
            if authenticated {
                debug!("auth succeded");

                break;
            } else {
                debug!("auth failed");
            }
        }

        if !authenticated {
            info!("all identities failed to authenticate");
            return Err(anyhow!("all identities failed to authenticate"));
        }

        let channel = session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await?;
        let sftp_session = SftpSession::new(channel.into_stream()).await?;
        info!(
            "GitDirs::connect sftp path: {:?}",
            sftp_session.canonicalize(".").await?
        );

        let this = Self {
            user: user.to_string(),
            host: host.to_string(),
            pending_dirs: VecDeque::from_iter(rootdirs.into_iter().map(|p| p.as_ref().to_string())),
            session,
            sftp_session,
            state: GitDirsState::Initial,
        };

        Ok(this)
    }
}

// #[tracing::instrument(level = "trace")]
// async fn read_subdirs<P>(&mut self, dir: P) -> Vec<String>
// where
//     P: Into<String> + Debug,
// {
//     let dir = dir.into();
//     if let Ok(rd) = self.sftp_session.read_dir(dir.as_str()).await {
//         rd.filter_map(|entry| {
//             entry
//                 .file_type()
//                 .is_dir()
//                 .then_some(entry.file_name())
//                 .map(|file_name| path_join(&dir, file_name))
//         })
//         .collect::<Vec<_>>()
//     } else {
//         Vec::default()
//     }
// }

// #[tracing::instrument(level = "trace")]
// async fn is_dir<P>(&self, path: P) -> bool
// where
//     P: Into<String> + Debug,
// {
//     self.sftp_session
//         .symlink_metadata(path)
//         .await
//         .is_ok_and(|m| m.is_dir())
// }

fn path_join<S1, S2>(p1: S1, p2: S2) -> String
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    Path::new(p1.as_ref())
        .join(Path::new(p2.as_ref()))
        .into_os_string()
        .into_string()
        .unwrap()
}

// impl Stream for GitDirs {
//     type Item = String;

//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut state = GitDirsState::Initial;
//         mem::swap(&mut self.state, &mut state);
//         match state {
//             GitDirsState::Initial => match self.pending_dirs.pop_front() {
//                 None => {
//                     trace!("GitDirs::Stream done");
//                     Poll::Ready(None)
//                 }
//                 Some(dir) => {
//                     let gitdir_candidate = path_join(&dir, ".git");
//                     let symlink_metadata = self.sftp_session.symlink_metadata(gitdir_candidate);
//                     let s = GitDirsState::IsGitDir(dir, Box::pin(symlink_metadata));
//                     self.state = s;
//                     cx.waker().wake_by_ref();
//                     Poll::Pending
//                 }
//             },
//             GitDirsState::IsGitDir(path, mut is_git_dir) => match is_git_dir.as_mut().poll(cx) {
//                 Poll::Ready(metadata) => {
//                     match metadata {
//                         Ok(metadata) => {
//                             if metadata.is_dir() {
//                                 trace!("GitDirs::Stream yield {:?}", &path);
//                                 Poll::Ready(Some(path))
//                             } else {
//                                 trace!("GitDirs::Stream read_subdirs {:?}", path);
//                                 let read_dir = self.sftp_session.read_dir(path);
//                                 let q = Box::pin(read_dir);
//                                 let mut state = GitDirsState::ReadSubdirs(q);
//                                 mem::swap(&mut self.state, &mut state);
//                                 cx.waker().wake_by_ref();
//                                 Poll::Pending
//                             }
//                         }
//                         Err(e) => {
//                             warn!("IsGitDir error {}", e);
//                             // TODO is the best we can do to end the stream?
//                             Poll::Ready(None)
//                         }
//                     }
//                 }
//                 Poll::Pending => {
//                     let mut state = GitDirsState::IsGitDir(path, is_git_dir);
//                     mem::swap(&mut self.state, &mut state);
//                     Poll::Pending
//                 }
//             },
//             GitDirsState::ReadSubdirs(mut read_subdirs) => match read_subdirs.as_mut().poll(cx) {
//                 Poll::Ready(subdirs) => match subdirs {
//                     Ok(subdirs) => {
//                         self.pending_dirs.extend(subdirs.filter_map(|entry| {
//                             if entry.file_type().is_dir() {
//                                 Some(entry.file_name())
//                             } else {
//                                 None
//                             }
//                         }));
//                         cx.waker().wake_by_ref();
//                         Poll::Pending
//                     }
//                     Err(e) => {
//                         warn!("ReadSubdirs error {}", e);
//                         // TODO is the best we can do to end the stream?
//                         Poll::Ready(None)
//                     }
//                 },
//                 Poll::Pending => {
//                     let mut state = GitDirsState::ReadSubdirs(read_subdirs);
//                     mem::swap(&mut self.state, &mut state);
//                     Poll::Pending
//                 }
//             },
//         }
//     }
// }

struct Client;

// cribbed from https://github.com/AspectUnk/russh-sftp/blob/master/examples/client.rs
#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        info!(
            "skipping check of server key: {} {} ",
            server_public_key.name(),
            server_public_key.fingerprint()
        );
        Ok(true)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        info!("{} bytes received on {:?}", data.len(), channel);
        Ok(())
    }
}
