use std::{collections::HashSet, fmt::Display, time::Instant};

use serde::{Deserialize, Serialize};

use crate::entries::Note;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Host(String); // ex: "misskey.io"

impl From<String> for Host {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    pub host: String,
    pub api_key: String,

    #[serde(default)]
    pub disable: bool,
}

#[derive(Debug, Clone)]
pub struct NoteEntry {
    pub host: Host,
    pub uri: String,
    pub note: Note,
    pub branches: HashSet<Branch>,
    pub inserted_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Branch {
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
