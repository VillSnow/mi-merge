use std::{collections::HashSet, fmt::Display, time::Instant};

use serde::{Deserialize, Serialize};

use crate::entries::Note;

mod dyn_note_model;
mod error;
mod note_key;
mod note_model;

pub use dyn_note_model::DynNoteModel;
pub use error::MiMergeError;
pub use note_key::NoteKey;
pub use note_model::NoteModel;

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
