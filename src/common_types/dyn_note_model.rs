use std::collections::HashSet;

use crate::mi_models::Note;

use super::{BranchKey, Host, NoteModel};

#[derive(Debug, Clone)]
pub struct DynNoteModel {
    pub original_host: Host,
    pub source_host: Host,
    pub uri: String,
    pub mi_note: Note,

    pub reactions: Vec<(String, i32)>,
    pub branches: HashSet<BranchKey>,
}

impl DynNoteModel {
    pub fn from_model(global_note: NoteModel) -> Self {
        Self {
            original_host: global_note.original_host,
            source_host: global_note.source_host,
            uri: global_note.uri,
            mi_note: global_note.mi_note,
            reactions: Default::default(),
            branches: Default::default(),
        }
    }
    pub fn from_mi_model(mi_note: Note, source_host: Host) -> Self {
        Self::from_model(NoteModel::from_mi_model(mi_note, source_host))
    }
}
