use std::collections::{HashMap, HashSet};

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::warn;

use crate::common_types::{BranchKey, DynNoteModel, MiMergeError, NoteModel};

#[derive(Debug, Default)]
pub struct ServerNoteRepo {
    notes: HashMap<String, NoteModel>,
    branches: HashMap<String, HashSet<BranchKey>>,
    reactions: HashMap<String, HashMap<String, i32>>,
    senders: Vec<UnboundedSender<DynNoteModel>>,
}

impl ServerNoteRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(
        &mut self,
        note: NoteModel,
        branches: HashSet<BranchKey>,
    ) -> Result<(), MiMergeError> {
        let note_id = note.mi_note.id.clone();

        self.notes.insert(note_id.clone(), note.clone());
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

        let mut dyn_model = DynNoteModel::from_model(note.clone());
        if let Some(xs) = self.reactions.get(note_id) {
            dyn_model
                .reactions
                .extend(xs.iter().map(|(k, &v)| (k.clone(), v)));
        }
        if let Some(xs) = self.branches.get(note_id) {
            dyn_model.branches.extend(xs.iter().cloned());
        }

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
