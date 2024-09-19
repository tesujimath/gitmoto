use serde::Deserialize;
use std::{env, fmt::Display, fs::read_to_string, io, path::PathBuf};
use tracing::debug;

#[derive(Default, Deserialize, Debug)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub display: DisplayConfig,
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
            Ok(config)
        }
        None => {
            debug!("no config file, using defaults");
            Ok(Config::default())
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    TomlDecode(toml::de::Error),
    XdgBaseDirectories(xdg::BaseDirectoriesError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Io(e) => write!(f, "I/O error {}", e),
            TomlDecode(e) => write!(f, "TOML decode error {}", e),
            XdgBaseDirectories(e) => write!(f, "XDG error {}", e),
        }
    }
}

impl std::error::Error for Error {}
