use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::warn;

use crate::common_types::{Branch, DynNoteModel, MiMergeError, NoteModel};

#[derive(Debug, Default)]
pub struct ServerNoteRepo {
    notes: HashMap<String, NoteModel>,
    branches: HashMap<String, HashSet<Branch>>,
    reactions: HashMap<String, HashMap<String, i32>>,
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
        let note_id = entry.note.id.clone();

        self.notes.insert(note_id.clone(), entry.clone());
        self.branches
            .entry(note_id.clone())
            .or_default()
            .extend(branches.into_iter());

        self.send_dyn_note(&note_id);

        Ok(())
    }

    pub fn incr_reaction(&mut self, note_id: &str, reaction: &str) {
        self.reactions
            .entry(note_id.to_owned())
            .or_default()
            .entry(reaction.to_owned())
            .and_modify(|x| *x += 1)
            .or_insert(1);

        self.send_dyn_note(note_id);
    }

    pub fn send_dyn_note(&self, note_id: &str) {
        let note = if let Some(note) = self.notes.get(note_id) {
            note
        } else {
            warn!("unknown note id");
            return;
        };

        let dyn_model = DynNoteModel {
            original_host: note.original_host.clone(),
            source_host: note.source_host.clone(),
            uri: note.uri.clone(),
            note: note.note.clone(),
            reactions: self
                .reactions
                .get(note_id)
                .map(|xs| xs.iter().map(|(k, &v)| (k.clone(), v)).collect())
                .unwrap_or_default(),
            branches: self.branches.get(note_id).cloned().unwrap_or_default(),
        };
        for tx in &self.senders {
            tx.send(dyn_model.clone()).expect("mpsc error");
        }
    }

    pub fn make_updated_note_receiver(&mut self) -> UnboundedReceiver<DynNoteModel> {
        let (tx, rx) = unbounded_channel();
        self.senders.push(tx);
        rx
    }
}
