use super::{MiMergeError, NoteModel};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteKey {
    pub uri: String,
}

impl NoteKey {
    pub fn from_note_entry(entry: &NoteModel) -> Result<NoteKey, MiMergeError> {
        let uri = if entry.source_host == entry.original_host {
            format!("https://{}/notes/{}", entry.original_host, entry.note.id)
        } else {
            entry
                .note
                .uri
                .as_ref()
                .ok_or(MiMergeError::InvalidNote)?
                .clone()
        };
        Ok(NoteKey { uri })
    }
}
