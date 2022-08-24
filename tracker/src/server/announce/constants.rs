/// String used for generaing the tracker's ID
pub const TRACKER_ID: &str = "Polleria Rustiseria Tracker ID :)";

/// Default amount of peers to be return in a Announce Request
pub const DEFAULT_NUMWANT: u32 = 50;

// Mandatory keys that need to be found in HTTP request query params
pub const INFO_HASH_KEY: &str = "info_hash";
pub const PEER_ID_KEY: &str = "peer_id";
pub const UPLOADED_KEY: &str = "uploaded";
pub const DOWNLOADED_KEY: &str = "downloaded";
pub const LEFT_KEY: &str = "left";
pub const PORT_KEY: &str = "port";