use std::collections::HashMap;
use std::env;
use std::fs;
use std::num;
use std::path;
use std::str;

const LISTEN_PORT: &str = "listen_port";
const LOG_PATH: &str = "log_path";
const DOWNLOAD_PATH: &str = "download_path";
const SEPARATOR: &str = "=";

#[allow(dead_code)]
#[derive(Debug)]
/// Configuration of the bittorrent client
pub struct Config {
    /// TCP port where client is receiving connections from other peers
    listen_port: u16,
    /// file path where logs will be written to
    log_path: String,
    /// file path where the downloaded file will be located at
    download_path: String,
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
    /// let config = Config::from_arguments(env::args());
    /// ```
    /// ```ignore
    /// assert_eq!(config.port, 4325);
    /// ```
    pub fn from_arguments(args: env::Args) -> Result<Config, ConfigError> {
        let mut args = args.skip(1);
        let config_path = args.next().ok_or(ConfigError::NoConfigPath)?;
        from_file(&config_path)
    }
}

fn from_file(path: &str) -> Result<Config, ConfigError> {
    let content =
        fs::read_to_string(path).map_err(|_| ConfigError::InvalidPath(path.to_string()))?;
    let lines = content.lines();
    let config_dictionary = create_config_dict(lines);
    let config = create_config(&config_dictionary)?;
    Ok(config)
}

fn create_config(config_dict: &HashMap<String, String>) -> Result<Config, ConfigError> {
    let listen_port = match config_dict.get(LISTEN_PORT) {
        Some(port) => port,
        None => return Err(ConfigError::MissingKey(LISTEN_PORT.to_string())),
    };
    let listen_port = listen_port
        .parse::<u16>()
        .map_err(ConfigError::InvalidPort)?;

    let log_path = match config_dict.get(LOG_PATH) {
        Some(path) => path,
        None => return Err(ConfigError::MissingKey(LOG_PATH.to_string())),
    };
    let download_path = match config_dict.get(DOWNLOAD_PATH) {
        Some(path) => path,
        None => return Err(ConfigError::MissingKey(DOWNLOAD_PATH.to_string())),
    };

    validate_path(download_path)?;
    validate_path(log_path)?;
    Ok(Config {
        listen_port,
        log_path: log_path.to_string(),
        download_path: download_path.to_string(),
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

#[derive(Debug, PartialEq)]
/// Errors that can occur when parsing the config file
pub enum ConfigError {
    /// The port is not in range or a valid number
    InvalidPort(num::ParseIntError),
    /// The path is not valid or exists
    InvalidPath(String),
    /// there is a key missing in the config file
    MissingKey(String),
    /// no config path was passed in arguments
    NoConfigPath,
}

// implement display for every type of error
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // ConfigError::IoError(e) => write!(f, "Config - IO error: {}", e),
            ConfigError::InvalidPort(e) => write!(f, "Config - Invalid port: {}", e),
            ConfigError::InvalidPath(e) => {
                write!(f, "Config - Could not find directory in path: {}", e)
            }
            ConfigError::MissingKey(key) => write!(f, "Config - Missing key: {}", key),
            ConfigError::NoConfigPath => write!(f, "Config - No config path specified"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parses_correct_config() {
        let config = from_file("src/config/test_files/correct_config.txt").unwrap();
        assert_eq!(config.listen_port, 4424);
        assert_eq!(config.log_path, "src/config/test_files/");
        assert_eq!(config.download_path, "src/config/test_files/");
    }

    #[test]
    fn throws_on_not_config_path_in_command_args() {
        let args = env::args();
        let config = Config::from_arguments(args);
        assert_eq!(config.unwrap_err(), ConfigError::NoConfigPath);
    }

    #[test]
    fn throws_on_invalid_config_path() {
        let config = from_file("34f^^f.-ªª");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::InvalidPath("34f^^f.-ªª".to_string())
        );
    }

    #[test]
    fn throws_on_non_existant_config_file() {
        let config = from_file("src/config/test_files/non_existant_config.txt");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::InvalidPath("src/config/test_files/non_existant_config.txt".to_string())
        );
    }

    #[test]
    fn throws_on_missing_keys() {
        let config = from_file("src/config/test_files/missing_download_path_config.txt");
        assert_eq!(
            config.unwrap_err(),
            ConfigError::MissingKey(DOWNLOAD_PATH.to_string())
        );
    }

    #[test]
    fn throws_on_invalid_port() {
        let config = from_file("src/config/test_files/invalid_port_config.txt");
        assert!(matches!(
            config,
            Err(ConfigError::InvalidPort(num::ParseIntError { .. }))
        ));
    }

    #[test]
    fn throws_on_invalid_path() {
        let config = from_file("src/config/test_files/wrong_path_log_config.txt");
        assert!(matches!(config, Err(ConfigError::InvalidPath(_))));
    }

    #[test]
    fn throws_on_invalid_format_config() {
        let config = from_file("src/config/test_files/invalid_format_config.txt");
        assert!(matches!(config, Err(ConfigError::MissingKey(_))));
    }
}
