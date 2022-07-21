use super::constants::*;
use super::types::TrackerEndpoint;

pub fn parse_path(path: &str) -> TrackerEndpoint {
    if path == ANNOUNCE_ENDPOINT {
        return TrackerEndpoint::Announce;
    } else if path == STATS_ENDPOINT {
        return TrackerEndpoint::Stats;
    } else if path == METRICS_ENDPOINT {
        return TrackerEndpoint::Metrics;
    }

    TrackerEndpoint::Other
}
