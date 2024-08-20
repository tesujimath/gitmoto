use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use russh::{
    client::{self, Session},
    keys::{agent::client::AgentClient, key},
    ChannelId,
};
use russh_sftp::client::SftpSession;
use std::{
    collections::VecDeque,
    fmt::Debug,
    future::Future,
    mem,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::fs::{read_dir, symlink_metadata};
use tokio_stream::wrappers::ReadDirStream;
use tracing::{debug, info, trace};

/// stream returning all git directories under `rootdir`
pub struct GitDirs {
    user: String,
    host: String,
    pending_dirs: VecDeque<PathBuf>,
    session: client::Handle<Client>,
    sftp_session: SftpSession,
    state: GitDirsState,
}

// futures we could be awaiting
enum GitDirsState {
    Initial,
    IsGitDir(PathBuf, Pin<Box<dyn Future<Output = bool>>>),
    QueueSubdirs(Pin<Box<dyn Future<Output = Vec<PathBuf>>>>),
}

impl GitDirs {
    pub async fn connect<S1, S2, I, P>(user: S1, host: S2, rootdirs: I) -> Result<Self>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let user = user.as_ref();
        let host = host.as_ref();

        let mut agent = AgentClient::connect_env().await?;
        let mut identities = agent.request_identities().await?;
        for (i, id) in identities.iter().enumerate() {
            debug!(
                "agent supplied identity {} {} {}",
                i,
                id.name(),
                id.fingerprint()
            );
        }

        let config = russh::client::Config::default();
        let sh = Client {};
        let mut session = russh::client::connect(Arc::new(config), (host, 22), sh).await?;

        let id = identities
            .pop()
            .ok_or(anyhow!("ssh agent has no identities"))?;

        // first try with agent auth
        let (_, authorized) = session.authenticate_future(user, id, agent).await;
        let authorized = authorized?;
        debug!("authenticate_future {:?}", authorized);

        if !authorized {
            return Err(anyhow!("failed to authorize ssh session"));
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
            pending_dirs: VecDeque::from_iter(
                rootdirs.into_iter().map(|p| p.as_ref().to_path_buf()),
            ),
            session,
            sftp_session,
            state: GitDirsState::Initial,
        };

        Ok(this)
    }
}

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
        info!("data on channel {:?}: {}", channel, data.len());
        Ok(())
    }
}
