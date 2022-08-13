use super::constants::*;
use super::endpoints::TrackerEndpoint;

pub fn parse_path(path: &str) -> TrackerEndpoint {
    // remove slash at the end
    let path = path.trim_end_matches('/');

    if path == ANNOUNCE_ENDPOINT {
        TrackerEndpoint::Announce
    } else if path == METRICS_ENDPOINT {
        TrackerEndpoint::Metrics
    } else {
        TrackerEndpoint::StaticResource
    }
}
