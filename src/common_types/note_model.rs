use crate::entries::Note;

use super::Host;

#[derive(Debug, Clone)]
pub struct NoteModel {
    pub original_host: Host,
    pub source_host: Host,
    pub uri: String,
    pub note: Note,
}

impl NoteModel {
    pub fn from_ws_model(ws_note: Note, source_host: Host) -> Self {
        let original_host = ws_note
            .user
            .host
            .clone()
            .map(Host::from)
            .unwrap_or(source_host.clone());

        let uri = ws_note
            .uri
            .clone()
            .unwrap_or(format!("https://{}/notes/{}", source_host, ws_note.id));

        Self {
            original_host,
            source_host,
            uri,
            note: ws_note,
        }
    }
}
