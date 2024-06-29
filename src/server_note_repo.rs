use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::common_types::{Branch, DynNoteModel, MiMergeError, NoteKey, NoteModel};

#[derive(Debug, Default)]
pub struct ServerNoteRepo {
    notes: HashMap<NoteKey, NoteModel>,
    branches: HashMap<NoteKey, HashSet<Branch>>,
    senders: Vec<UnboundedSender<DynNoteModel>>,
}

impl ServerNoteRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn upsert(
        &mut self,
        entry: NoteModel,
        branches: HashSet<Branch>,
    ) -> Result<(), MiMergeError> {
        let key = NoteKey::from_note_entry(&entry)?;

        self.notes.insert(key.clone(), entry.clone());
        self.branches
            .entry(key.clone())
            .or_default()
            .extend(branches.into_iter());

        let dyn_model = DynNoteModel {
            original_host: entry.original_host.clone(),
            source_host: entry.source_host.clone(),
            uri: entry.uri.clone(),
            note: entry.note.clone(),
            branches: self.branches[&key].clone(),
        };
        for tx in &self.senders {
            tx.send(dyn_model.clone()).expect("mpsc error");
        }

        Ok(())
    }

    pub fn make_updated_note_receiver(&mut self) -> UnboundedReceiver<DynNoteModel> {
        let (tx, rx) = unbounded_channel();
        self.senders.push(tx);
        rx
    }
}
