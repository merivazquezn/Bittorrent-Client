#[derive(Debug)]
pub enum TrackerEndpoint {
    Announce,
    StaticResource,
    Metrics,
    Torrents,
}
