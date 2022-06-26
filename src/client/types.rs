use crate::config::Config;
use crate::metainfo::Metainfo;
use crate::tracker::TrackerService;

pub struct Client {
    pub peer_id: [u8; 20],
    pub config: Config,
    pub metainfo: Metainfo,
    pub tracker_service: TrackerService,
}
