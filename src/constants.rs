use std::time::Duration;

pub const BLOCK_SIZE: u32 = 16 * u32::pow(2, 10);
pub const TIME_BETWEEN_ACCEPTS: Duration = Duration::from_millis(100);
