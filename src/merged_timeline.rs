use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    ops::DerefMut,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::common_types::{DynNoteModel, MiMergeError, NoteKey};

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
struct ColumnEntry {
    dyn_note_model: Arc<RwLock<DynNoteModel>>,
    inserted_at: Instant,
}

#[derive(Debug, Default)]
pub struct MergedTimeline {
    column: VecDeque<ColumnEntry>,
    dictionary: HashMap<NoteKey, Arc<RwLock<DynNoteModel>>>,
    column_senders: Vec<UnboundedSender<Vec<DynNoteModel>>>,
}

impl MergedTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn upsert(&mut self, mut incoming: DynNoteModel) -> Result<(), MiMergeError> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let key = NoteKey {
            uri: incoming.uri.clone(),
        };

        match self.dictionary.entry(key) {
            Occupied(current) => {
                let mut current = current.get().write().await;

                // 次の優先順位で `self.dictionary` に格納する。
                // 1. ソースホストとオリジナルホストが同じもの。
                // 2. 新しく来たもの。
                if current.source_host != current.original_host
                    || incoming.source_host == incoming.original_host
                {
                    std::mem::swap(current.deref_mut(), &mut incoming);
                }

                current.branches.extend(incoming.branches.into_iter());
            }
            Vacant(entry) => {
                let now = Instant::now();

                let incoming = Arc::new(RwLock::new(incoming));
                entry.insert(incoming.clone());
                insert_into_column(&mut self.column, incoming.clone(), now).await;
            }
        };

        let mut sending_item = Vec::new();
        for x in &self.column {
            sending_item.push(x.dyn_note_model.read().await.clone());
        }

        for sender in &self.column_senders {
            sender.send(sending_item.clone()).expect("mpsc error");
        }

        Ok(())
    }

    pub fn make_column_receiver(&mut self) -> UnboundedReceiver<Vec<DynNoteModel>> {
        let (tx, rx) = unbounded_channel();
        self.column_senders.push(tx);
        rx
    }
}

async fn insert_into_column(
    column: &mut VecDeque<ColumnEntry>,
    incoming: Arc<RwLock<DynNoteModel>>,
    now: Instant,
) {
    let sort_limit_dur = Duration::from_millis(500);

    let inserting_created_at = incoming.read().await.note.created_at.clone();

    let mut n = 0;
    for i in 0..column.len() {
        let exists = column[i].dyn_note_model.read().await;
        if inserting_created_at >= exists.note.created_at
            || now.duration_since(column[i].inserted_at) > sort_limit_dur
        {
            n = i;
            break;
        }
    }
    column.insert(
        n,
        ColumnEntry {
            dyn_note_model: incoming,
            inserted_at: now,
        },
    );
}
