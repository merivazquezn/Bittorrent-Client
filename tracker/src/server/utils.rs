use super::constants::*;
use super::endpoints::TrackerEndpoint;

pub fn parse_path(path: &str) -> TrackerEndpoint {
    if path == ANNOUNCE_ENDPOINT {
        TrackerEndpoint::Announce
    } else if path == METRICS_ENDPOINT {
        TrackerEndpoint::Metrics
    } else {
        TrackerEndpoint::StaticResource
    }
}
