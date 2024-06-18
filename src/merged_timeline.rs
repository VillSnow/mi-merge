use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{
    common_types::{Branch, Host, NoteEntry},
    entries::Note,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteKey {
    pub created_at: DateTime<Utc>,
    pub host: Host,
    pub uri: String,
}

#[derive(Debug)]
pub enum MergedTimeLineError {
    InvalidNote,
}

impl std::fmt::Display for MergedTimeLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MergedTimeLineError::InvalidNote => {
                write!(f, "invalid note")
            }
        }
    }
}

impl Error for MergedTimeLineError {}

#[derive(Debug, Default)]
pub struct MergedTimeline {
    column: VecDeque<Arc<RwLock<NoteEntry>>>,
    dictionary: HashMap<NoteKey, Arc<RwLock<NoteEntry>>>,
    column_senders: Vec<UnboundedSender<Vec<NoteEntry>>>,
}

impl NoteKey {
    fn from_note(host: &Host, note: &Note) -> Result<NoteKey, MergedTimeLineError> {
        let note_host = note
            .user
            .host
            .as_ref()
            .map(|s| Host::from(s.clone()))
            .unwrap_or(host.clone());

        let uri = if &note_host == host {
            format!("https://{}/notes/{}", host, note.id)
        } else {
            note.uri
                .as_ref()
                .ok_or(MergedTimeLineError::InvalidNote)?
                .clone()
        };
        Ok(NoteKey {
            created_at: DateTime::from_str(&note.created_at)
                .map_err(|_| MergedTimeLineError::InvalidNote)?,
            host: note_host.clone(),
            uri,
        })
    }
}

impl MergedTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn insert(
        &mut self,
        host: Host,
        branches: HashSet<Branch>,
        note: Note,
    ) -> Result<(), MergedTimeLineError> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let key = NoteKey::from_note(&host, &note)?;
        let note_host = key.host.clone();
        let note_uri = key.uri.clone();

        match self.dictionary.entry(key) {
            Occupied(dict_entry) => {
                let mut entry = dict_entry.get().write().await;
                if dict_entry.key().host == host {
                    entry.note = note.clone();
                }
                entry.branches.extend(branches.clone().into_iter());
            }
            Vacant(dict_entry) => {
                let now = Instant::now();

                let inserting_created_at = note.created_at.clone();
                let entry = Arc::new(RwLock::new(NoteEntry {
                    host: note_host,
                    uri: note_uri,
                    note,
                    branches,
                    inserted_at: now.clone(),
                }));

                let sort_limit_dur = Duration::from_millis(500);

                let mut n = self.column.len();
                for i in 0..self.column.len() {
                    let exists = self.column[i].read().await;
                    if inserting_created_at >= exists.note.created_at
                        || now.duration_since(exists.inserted_at) > sort_limit_dur
                    {
                        n = i;
                        break;
                    }
                }
                self.column.insert(n, entry.clone());

                dict_entry.insert(entry);
            }
        }

        let mut next_value = Vec::new();
        for x in &self.column {
            next_value.push(x.read().await.deref().clone());
        }
        for tx in &self.column_senders {
            tx.send(next_value.clone()).expect("mpsc error");
        }

        Ok(())
    }

    pub fn make_column_receiver(&mut self) -> UnboundedReceiver<Vec<NoteEntry>> {
        let (tx, rx) = unbounded_channel();
        self.column_senders.push(tx);
        rx
    }
}
