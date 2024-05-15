mod color_parser;
mod config;
mod config_loader;
pub mod keysyms;

pub use config::{AvailableActions, Config};
use config_loader::{ConfigError, UnresolvedConfig};
use std::path::{Path, PathBuf};

static APP_NAME: &str = "lucky";
static CONFIG_FILE: &str = "config.toml";
static XDG_HOME: &str = "XDG_CONFIG_HOME";
static LUCKY_CONF_ENV_VAR: &str = "LUCKY_CONFIG";

/// Verify if `XDG_HOME`/.config/lucky/config.toml exists
fn get_config_dir_path() -> Option<PathBuf> {
    let var = match std::env::var(XDG_HOME) {
        Ok(home_path) => Some(
            Path::new(&home_path)
                .join(".config")
                .join(APP_NAME)
                .join(CONFIG_FILE),
        ),
        Err(_) => None,
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
            ConfigError::InvalidKey(msg) => anyhow::bail!(msg),
            ConfigError::InvalidWorkspaces(msg) => anyhow::bail!(msg),
            ConfigError::InvalidBorderWidth(msg) => anyhow::bail!(msg),
            ConfigError::InvalidBorderColor(msg) => anyhow::bail!(msg),
            ConfigError::InvalidColor(msg) => anyhow::bail!(msg),
        },
    }
}

/// Try to load the configuration from 3 places, in the following order:
///
/// * If set, `LUCKY_CONFIG` will be prioritized and the config will be loaded from there;
/// * If not, will attempt to load from `XDG_HOME`/.config/lucky/config.toml;
/// * If not present on any of the directories above, will load the default configuration;
pub fn load_config() -> Config {
    let config_path = match std::env::var(LUCKY_CONF_ENV_VAR) {
        Ok(var) => Some(PathBuf::from(&var).join(CONFIG_FILE)),
        Err(_) => get_config_dir_path(),
    };
    match config_path
        .map(load_config_from_file)
        .unwrap_or(Ok(Config::default()))
    {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("{e:?}");
            Config::default()
        }
    }
}
