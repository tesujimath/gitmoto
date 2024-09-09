use anyhow::{anyhow, Result};
use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use globset::GlobSet;
use russh::{
    client,
    keys::{agent::client::AgentClient, key},
    ChannelId,
};
use russh_sftp::client::SftpSession;
use std::{collections::VecDeque, fmt::Debug, path::Path, sync::Arc};
use tracing::{debug, info};

pub struct Connection {
    _session: client::Handle<Client>,
    sftp_session: SftpSession,
}

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection")
    }
}

async fn connect<S1, S2>(user: S1, host: S2) -> Result<Connection>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
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

    Ok(Connection {
        _session: session,
        sftp_session,
    })
}

impl Connection {
    #[tracing::instrument(level = "trace")]
    async fn read_subdirs<P>(&self, dir: P) -> Vec<String>
    where
        P: Into<String> + Debug,
    {
        let dir = dir.into();
        if let Ok(rd) = self.sftp_session.read_dir(dir.as_str()).await {
            rd.filter_map(|entry| {
                entry
                    .file_type()
                    .is_dir()
                    .then_some(entry.file_name())
                    .map(|file_name| path_join(&dir, file_name))
            })
            .collect::<Vec<_>>()
        } else {
            Vec::default()
        }
    }

    #[tracing::instrument(level = "trace")]
    async fn is_dir<P>(&self, path: P) -> bool
    where
        P: Into<String> + Debug,
    {
        self.sftp_session
            .symlink_metadata(path)
            .await
            .is_ok_and(|m| m.is_dir())
    }
}

pub fn git_dirs<S1, S2, I, P>(
    user: S1,
    host: S2,
    rootdirs: I,
    _excludes: GlobSet,
) -> impl Stream<Item = String>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    I: IntoIterator<Item = P>,
    P: AsRef<str>,
{
    let mut pending_dirs =
        VecDeque::from_iter(rootdirs.into_iter().map(|dir| dir.as_ref().to_string()));

    stream! {
        let connection = connect(user, host).await.unwrap(); // TODO return error, or restructure to separate connect from stream

        while let Some(dir)  = pending_dirs.pop_front() {
            if dir.ends_with(".git") {
                yield dir;
            } else {
            let gitdir_candidate = path_join(&dir, ".git");

            if connection.is_dir(gitdir_candidate).await {
                yield dir;
            } else {
                let subdirs = connection.read_subdirs(&dir).await;

                pending_dirs.extend(subdirs);
            }
            }
        }
    }
}

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
        _channel: ChannelId,
        _data: &[u8],
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        // debug!("{} bytes received on {:?}", data.len(), channel);
        Ok(())
    }
}
