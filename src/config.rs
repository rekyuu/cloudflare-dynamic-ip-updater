use merge::Merge;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use log::{debug, info, warn};

use crate::constants::*;

#[derive(Serialize, Deserialize, Merge, Clone)]
pub struct GeneralConfig {
    pub(crate) wait_duration: Option<u64>
}

#[derive(Serialize, Deserialize, Merge, Clone)]
pub struct CloudflareConfig {
    pub(crate) zone_id: Option<String>,
    pub(crate) api_token: Option<String>,
    pub(crate) dns_record_id: Option<String>,
}

#[derive(Serialize, Deserialize, Merge, Clone)]
pub struct Config {
    pub(crate) general: Option<GeneralConfig>,
    pub(crate) cloudflare: Option<CloudflareConfig>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        GeneralConfig {
            wait_duration: Some(DEFAULT_WAIT_TIME)
        }
    }
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        CloudflareConfig {
            zone_id: Some(DEFAULT_NOT_SET.to_string()),
            api_token: Some(DEFAULT_NOT_SET.to_string()),
            dns_record_id: Some(DEFAULT_NOT_SET.to_string())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: Some(GeneralConfig::default()),
            cloudflare: Some(CloudflareConfig::default()),
        }
    }
}

impl Config {
    pub fn load() -> Config {
        let dir = Config::get_config_dir();
        let filepath = dir.join(CONFIG_FILE_NAME);

        if !filepath.exists() {
            debug!("Creating default config.");
            Config::create_default_config_file()
                .expect("Unable to create default config file.");

            info!("Default configuration file created at {}.\nPlease fill it out and restart.", filepath.display());

            std::process::exit(0);
        }

        debug!("Config exists at {}, attempting to load.", filepath.display());
        let config_file = fs::File::open(&filepath).unwrap_or_else(|_| {
            panic!("Unable to load config file: {}", filepath.display());
        });

        let mut reader = BufReader::new(config_file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)
            .expect("Unable to read config file. Is the encoding UTF-8?");

        let config = toml::from_str::<Config>(contents.as_str())
            .expect("Unable to parse config file. Is it a valid TOML?")
            .merge_custom(Config::default());

        let cloudflare_config = config.cloudflare.as_ref().unwrap();

        if cloudflare_config.api_token.as_ref().unwrap() == DEFAULT_NOT_SET
            || cloudflare_config.zone_id.as_ref().unwrap() == DEFAULT_NOT_SET
            || cloudflare_config.dns_record_id.as_ref().unwrap() == DEFAULT_NOT_SET {
            warn!("Please ensure all values are configured in the configuration file located at {} and restart.", filepath.display());

            std::process::exit(0);
        }

        config
    }

    /// Initializes the default configuration file.
    fn create_default_config_file() -> Result<(), std::io::Error> {
        let dir = Config::get_config_dir();
        let filepath = dir.join(CONFIG_FILE_NAME);
        let config = Config::default();

        fs::create_dir_all(Config::get_config_dir())?;
        let mut config_file_path = fs::File::create(filepath)?;

        config_file_path.write_all(
            toml::to_string(&config)
                .unwrap()
                .as_bytes())
            .unwrap();

        Ok(())
    }

    /// Returns the configuration directory.
    fn get_config_dir() -> PathBuf {
        match dirs::config_dir() {
            Some(dir) => {
                dir.join(Path::new(CONFIG_FOLDER_NAME))
            },
            None => {
                dirs::config_dir()
                    .expect("Cannot get config folder or home directory.")
                    .join(format!(".{}", CONFIG_FOLDER_NAME))
            }
        }
    }

    /// Custom merge for the Config object and it's children.
    fn merge_custom(mut self, other: Config) -> Self {
        self.merge(other.clone());

        let mut cloudflare = self.cloudflare.unwrap();
        cloudflare.merge(other.cloudflare.unwrap());
        self.cloudflare = Some(cloudflare);

        self
    }
}