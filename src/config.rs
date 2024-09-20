use globset::{Glob, GlobSet};
use serde::{de, Deserialize, Deserializer};
use std::{env, fmt::Display, fs::read_to_string, io, path::PathBuf};
use tracing::debug;

#[derive(Default, Deserialize, Debug)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub filesystem: FilesystemConfig,
    pub display: DisplayConfig,
}

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct FilesystemConfig {
    pub scanner: FilesystemScannerConfig,
}

#[derive(Clone, Default, Deserialize, Debug)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct FilesystemScannerConfig {
    pub roots: Vec<String>,
    #[serde(deserialize_with = "deserialize_globset")]
    pub excludes: GlobSet,
}

#[derive(Default, Deserialize, Debug)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct DisplayConfig {
    #[serde(default = "default_collapse_paths")]
    pub collapse_paths: bool,
}

fn default_collapse_paths() -> bool {
    true
}

fn deserialize_globset<'de, D>(deserializer: D) -> Result<GlobSet, D::Error>
where
    D: Deserializer<'de>,
{
    let mut globset_builder = GlobSet::builder();
    for glob_str in Vec::<String>::deserialize(deserializer)? {
        globset_builder.add(Glob::new(&shellexpand::tilde(&glob_str)).map_err(de::Error::custom)?);
    }
    globset_builder.build().map_err(de::Error::custom)
}

pub fn read_config() -> Result<Config, Error> {
    const CONFIG_FILE: &str = "GITMOTO_CONFIG";
    let xdg_dirs = xdg::BaseDirectories::new().map_err(Error::XdgBaseDirectories)?;
    match env::var(CONFIG_FILE)
        .map(PathBuf::from)
        .ok()
        .or_else(|| xdg_dirs.find_config_file(CONFIG_FILE))
    {
        Some(config_path) => {
            debug!("reading config from {:?}", &config_path);

            let raw_config = read_to_string(&config_path).map_err(Error::Io)?;
            let config: Config = toml::from_str(&raw_config).map_err(Error::TomlDecode)?;

            valid_config(config)
        }
        None => {
            debug!("no config file, using defaults");
            Ok(Config::default())
        }
    }
}

fn valid_config(c: Config) -> Result<Config, Error> {
    if c.filesystem.scanner.roots.is_empty() {
        Err(Error::EmptyFilesystemScannerRoots)?
    }

    Ok(c)
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    TomlDecode(toml::de::Error),
    XdgBaseDirectories(xdg::BaseDirectoriesError),
    EmptyFilesystemScannerRoots,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Io(e) => write!(f, "I/O error {}", e),
            TomlDecode(e) => write!(f, "TOML decode error {}", e),
            XdgBaseDirectories(e) => write!(f, "XDG error {}", e),
            EmptyFilesystemScannerRoots => f.write_str("missing filesystem scanner roots"),
        }
    }
}

impl std::error::Error for Error {}
