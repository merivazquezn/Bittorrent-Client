use super::errors::ConfigError;
use log::*;
use std::collections::HashMap;
use std::fs;
use std::path;
use std::str;
const LISTEN_PORT: &str = "listen_port";
const LOG_PATH: &str = "log_path";
const DOWNLOAD_PATH: &str = "download_path";
const SEPARATOR: &str = "=";
use crate::logger::CustomLogger;
const LOGGER: CustomLogger = CustomLogger::init("Config");

#[allow(dead_code)]
#[derive(Debug)]
/// Configuration of the bittorrent client
pub struct Config {
    /// TCP port where client is receiving connections from other peers
    pub listen_port: u16,
    /// file path where logs will be written to
    pub log_path: String,
    /// file path where the downloaded file will be located at
    pub download_path: String,
}

impl Config {
    /// parses the command line arguments into the config
    ///
    /// # Returns Err
    ///
    /// the parsing will return Err if there are not enough arguments or they are invalid
    ///
    /// # Example
    ///
    /// ```
    /// use std::env;
    /// use bittorrent_rustico::config::Config;
    /// use std::path::PathBuf;
    /// let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    /// d.push("src/config/test_files/correct_config.txt");
    /// let path = d.into_os_string().into_string().unwrap();
    /// let config = Config::from_path(&path).unwrap();
    /// assert_eq!(config.listen_port, 4424);
    /// ```
    pub fn from_path(path: &str) -> Result<Config, ConfigError> {
        LOGGER.info(format!("Reading config file from path: {}", path));
        let content =
            fs::read_to_string(path).map_err(|_| ConfigError::InvalidPath(path.to_string()))?;
        let lines = content.lines();
        let config_dictionary = create_config_dict(lines);
        let config = create_config(&config_dictionary)?;
        Ok(config)
    }
}

fn create_config(config_dict: &HashMap<String, String>) -> Result<Config, ConfigError> {
    let listen_port = config_dict
        .get(LISTEN_PORT)
        .ok_or_else(|| ConfigError::MissingKey(LISTEN_PORT.to_string()))?
        .parse()?;
    trace!("Found listen_port: {}", listen_port);
    let log_path = config_dict
        .get(LOG_PATH)
        .ok_or_else(|| ConfigError::MissingKey(LOG_PATH.to_string()))?;
    trace!("Found log_path: {}", log_path);
    let download_path = config_dict
        .get(DOWNLOAD_PATH)
        .ok_or_else(|| ConfigError::MissingKey(DOWNLOAD_PATH.to_string()))?;
    trace!("Found download_path: {}", download_path);
    trace!("Validating paths exist");
    validate_path(download_path)?;
    validate_path(log_path)?;
    Ok(Config {
        listen_port,
        log_path: log_path.into(),
        download_path: download_path.into(),
    })
}

//validates that path point to valid directories
fn validate_path(path: &str) -> Result<(), ConfigError> {
    if !path::Path::new(path).exists() {
        return Err(ConfigError::InvalidPath(path.to_string()));
    }
    Ok(())
}

fn create_config_dict(lines: str::Lines) -> HashMap<String, String> {
    let mut config_dict: HashMap<String, String> = HashMap::new();
    lines.for_each(|line| {
        let mut split = line.split(SEPARATOR);
        if let Some(key) = split.next() {
            if let Some(value) = split.next() {
                config_dict.insert(key.to_string(), value.to_string());
            }
        }
    });
    config_dict
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parses_correct_config() {
        let config = Config::from_path("src/config/test_files/correct_config.txt").unwrap();
        assert_eq!(config.listen_port, 4424);
        assert_eq!(config.log_path, "src/config/test_files/");
        assert_eq!(config.download_path, "src/config/test_files/");
    }

    #[test]
    fn throws_on_not_config_path() {
        let config = Config::from_path("");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::InvalidPath("".to_string())
        );
    }

    #[test]
    fn throws_on_invalid_config_path() {
        let config = Config::from_path("34f^^f.-ªª");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::InvalidPath("34f^^f.-ªª".to_string())
        );
    }

    #[test]
    fn throws_on_non_existant_config_file() {
        let config = Config::from_path("src/config/test_files/non_existant_config.txt");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::InvalidPath("src/config/test_files/non_existant_config.txt".to_string())
        );
    }

    #[test]
    fn throws_on_missing_keys() {
        let config = Config::from_path("src/config/test_files/missing_download_path_config.txt");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::MissingKey(DOWNLOAD_PATH.to_string())
        );
    }

    #[test]
    fn throws_on_invalid_port() {
        let config = Config::from_path("src/config/test_files/invalid_port_config.txt");
        assert!(matches!(
            config,
            Err(ConfigError::InvalidPort(std::num::ParseIntError { .. }))
        ));
    }

    #[test]
    fn throws_on_invalid_path() {
        let config = Config::from_path("src/config/test_files/wrong_path_log_config.txt");
        assert!(matches!(config, Err(ConfigError::InvalidPath(_))));
    }

    #[test]
    fn throws_on_invalid_format_config() {
        let config = Config::from_path("src/config/test_files/invalid_format_config.txt");
        assert!(matches!(config, Err(ConfigError::MissingKey(_))));
    }
}
