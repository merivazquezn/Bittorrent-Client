/// Amount of worker threads to use.

/// Localhost ip address
pub const LOCALHOST: &str = "127.0.0.1";

/// Directory where the client store the downloaded pieces
pub const PIECES_DIR: &str = "./downloads/pieces";

/// Directory where the server store the logs
pub const LOGS_DIR: &str = "./logs";

/// Filename where the server logger store the logs
pub const SERVER_LOG_FILE_NAME: &str = "server_log.txt";

/// Timeout for the server read operation
pub const SERVER_READ_TIMEOUT: u64 = 100;

/// Timeout for the server write operation
pub const SERVER_WRITE_TIMEOUT: u64 = 100;
