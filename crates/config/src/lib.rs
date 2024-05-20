mod color_parser;
mod config;
mod config_loader;
pub mod keysyms;

pub use config::{AutoCommand, AvailableActions, Config};
use config_loader::{ConfigError, UnresolvedConfig};
use std::path::{Path, PathBuf};

static APP_NAME: &str = "lucky";
static CONFIG_FILE: &str = "config.toml";
static XDG_HOME: &str = "HOME";
static XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
static XDG_DATA_DIR: &str = "XDG_DATA_HOME";
static LUCKY_CONF_ENV_VAR: &str = "LUCKY_CONFIG";

/// Verify if `$HOME`/.config/lucky/config.toml exists
fn get_config_dir_path() -> Option<PathBuf> {
    let var = match std::env::var(XDG_CONFIG_HOME) {
        Ok(config_path) => {
            tracing::debug!(
                "loading config file from $XDG_CONFIG_HOME: {config_path}/lucky/config.toml"
            );
            Some(Path::new(&config_path).join(APP_NAME).join(CONFIG_FILE))
        }
        Err(_) => match std::env::var(XDG_HOME) {
            Ok(home_path) => {
                tracing::debug!(
                    "loading config file from $HOME: {home_path}/.config/lucky/config.toml"
                );
                Some(
                    Path::new(&home_path)
                        .join(".config")
                        .join(APP_NAME)
                        .join(CONFIG_FILE),
                )
            }

            Err(_) => {
                tracing::debug!("no config file found, loading default");
                None
            }
        },
    };
    var
}

fn load_config_from_file<P>(path: P) -> anyhow::Result<Config>
where
    P: AsRef<Path>,
{
    let config_file = std::fs::read_to_string(path.as_ref())?;
    let config = toml::from_str::<UnresolvedConfig>(&config_file)?;
    match Config::try_from(config) {
        Ok(config) => Ok(config),
        Err(e) => match e {
            ConfigError::Key(msg) => anyhow::bail!(msg),
            ConfigError::Workspaces(msg) => anyhow::bail!(msg),
            ConfigError::BorderWidth(msg) => anyhow::bail!(msg),
            ConfigError::BorderColor(msg) => anyhow::bail!(msg),
            ConfigError::InvalidCommand(msg) => anyhow::bail!(msg),
            ConfigError::Color(msg) => anyhow::bail!(msg),
        },
    }
}

/// Try to load the configuration from 3 places, in the following order:
///
/// * If set, `LUCKY_CONFIG` will be prioritized and the config will be loaded from there;
/// * If not available, will attempt to load from `XDG_CONFIG_HOME/lucky/config.toml`;
/// * If not available, will attempt to load from `HOME`/.config/lucky/config.toml;
/// * If not present on any of the directories above, will load the default configuration;
pub fn load_config() -> Config {
    let config_path = match std::env::var(LUCKY_CONF_ENV_VAR) {
        Ok(var) => {
            tracing::debug!("loading config file from $LUCKY_CONFIG: {var:?}");
            Some(PathBuf::from(&var).join(CONFIG_FILE))
        }
        Err(_) => get_config_dir_path(),
    };
    let config = match config_path
        .map(load_config_from_file)
        .unwrap_or(Ok(Config::default()))
    {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("{e:?}");
            Config::default()
        }
    };

    tracing::debug!("loaded config with {} actions", config.actions.len());
    tracing::debug!("loaded config with {} commands", config.commands.len());

    config
}

fn data_dir() -> anyhow::Result<PathBuf> {
    let data_path = match std::env::var(XDG_DATA_DIR) {
        Ok(data_path) => PathBuf::from(data_path).join(APP_NAME),
        Err(_) => match std::env::var(XDG_HOME) {
            Ok(home_path) => {
                let data_path = PathBuf::from(home_path)
                    .join(".local")
                    .join("share")
                    .join(APP_NAME);
                if !data_path.is_dir() {
                    std::fs::create_dir(&data_path)?;
                }
                data_path
            }
            Err(_) => anyhow::bail!("failed to get $HOME environment variable"),
        },
    };

    Ok(data_path)
}

pub fn log_file() -> anyhow::Result<(PathBuf, String)> {
    Ok((data_dir()?, format!("{}.log", APP_NAME)))
}
