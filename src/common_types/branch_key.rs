use super::Host;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchKey {
    pub host: Host,
    pub timeline: BranchTimeline,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BranchTimeline {
    Home,
    Local,
    Channel(String),
    Antenna(String),
}
