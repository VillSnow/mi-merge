use std::collections::HashSet;

use crate::entries::Note;

use super::{Branch, Host};

#[derive(Debug, Clone)]
pub struct DynNoteModel {
    pub original_host: Host,
    pub source_host: Host,
    pub uri: String,
    pub note: Note,

    pub reactions: Vec<(String, i32)>,
    pub branches: HashSet<Branch>,
}

impl DynNoteModel {
    pub fn from_ws_entity(ws_model: Note, source_host: Host) -> Self {
        let original_host = ws_model
            .user
            .host
            .clone()
            .map(Host::from)
            .unwrap_or(source_host.clone());

        let uri = ws_model
            .uri
            .clone()
            .unwrap_or(format!("https://{}/notes/{}", source_host, ws_model.id));

        Self {
            original_host,
            source_host,
            uri,
            note: ws_model,
            reactions: Vec::new(),
            branches: HashSet::new(),
        }
    }
}
