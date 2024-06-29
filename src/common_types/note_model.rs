use crate::mi_models::Note;

use super::Host;

#[derive(Debug, Clone)]
pub struct NoteModel {
    pub original_host: Host,
    pub source_host: Host,
    pub uri: String,
    pub mi_note: Note,
}

impl NoteModel {
    pub fn from_mi_model(mi_note: Note, source_host: Host) -> Self {
        let original_host = mi_note
            .user
            .host
            .clone()
            .map(Host::from)
            .unwrap_or(source_host.clone());

        let uri = mi_note
            .uri
            .clone()
            .unwrap_or(format!("https://{}/notes/{}", source_host, mi_note.id));

        Self {
            original_host,
            source_host,
            uri,
            mi_note,
        }
    }
}
