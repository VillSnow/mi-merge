use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
    future::Future,
    str::FromStr,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{common_types::Host, entries::Note, subject::SubjectMut};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteKey {
    pub created_at: DateTime<Utc>,
    pub host: Host,
    pub id: String,
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

#[derive(Debug)]
struct Entry {
    note: Note,
    branches: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct MergedTimeline {
    column: VecDeque<Arc<RwLock<Entry>>>,
    dictionary: HashMap<NoteKey, Arc<RwLock<Entry>>>,
    column_senders: Vec<UnboundedSender<Vec<Note>>>,
}

impl NoteKey {
    fn from_note(host: &Host, note: &Note) -> Result<NoteKey, MergedTimeLineError> {
        let note_host = note
            .user
            .host
            .as_ref()
            .map(|s| Host::from(s.clone()))
            .unwrap_or(host.clone());

        let original_id = if &note_host == host {
            note.id.clone()
        } else {
            let uri = note.uri.as_ref().ok_or(MergedTimeLineError::InvalidNote)?;
            uri.rsplit("/")
                .next()
                .ok_or(MergedTimeLineError::InvalidNote)?
                .to_owned()
        };
        Ok(NoteKey {
            created_at: DateTime::from_str(&note.created_at)
                .map_err(|_| MergedTimeLineError::InvalidNote)?,
            host: note_host.clone(),
            id: original_id,
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
        branches: HashSet<String>,
        note: Note,
    ) -> Result<(), MergedTimeLineError> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let key = NoteKey::from_note(&host, &note)?;

        match self.dictionary.entry(key) {
            Occupied(dict_entry) => {
                let mut entry = dict_entry.get().write().await;
                if dict_entry.key().host == host {
                    entry.note = note.clone();
                }
                entry.branches.extend(branches.clone().into_iter());
            }
            Vacant(dict_entry) => {
                let entry = Arc::new(RwLock::new(Entry { note, branches }));
                self.column.push_front(entry.clone());
                dict_entry.insert(entry);
            }
        }

        let mut next_value = Vec::new();
        for x in &self.column {
            next_value.push(x.read().await.note.clone());
        }
        for tx in &self.column_senders {
            tx.send(next_value.clone()).expect("mpsc error");
        }

        Ok(())
    }

    pub fn make_column_receiver(&mut self) -> UnboundedReceiver<Vec<Note>> {
        let (tx, rx) = unbounded_channel();
        self.column_senders.push(tx);
        rx
    }
}
