/// Amount of worker threads to use.
pub const POOL_WORKERS: usize = 5;

/// Localhost ip address
pub const LOCALHOST: &str = "127.0.0.0";

/// Directory where the client store the downloaded pieces
pub const PIECES_DIR: &str = "./downloads";

/// Directory where the server store the logs
pub const LOGS_DIR: &str = "./logs";

/// Filename where the server logger store the logs
pub const SERVER_LOG_FILE_NAME: &str = "server_log.txt";

/// Timeout for the server read operation
pub const SERVER_READ_TIMEOUT: u64 = 120;

/// Timeout for the server write operation
pub const SERVER_WRITE_TIMEOUT: u64 = 10;
