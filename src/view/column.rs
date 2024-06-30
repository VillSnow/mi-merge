use std::ops::Deref;

use dioxus::prelude::*;

use super::*;

#[derive(Clone, PartialEq, Eq, Props)]
pub struct ColumnProps {
    pub notes: Signal<Vec<NoteProps>>,
}

#[component]
pub fn Column(props: ColumnProps) -> Element {
    rsx! {
        div {
            for note in props.notes.read().deref() {
                article { key: "{note.uri}",
                    Note {
                        original_host: note.original_host.clone(),
                        uri: &note.uri,
                        avatar_url: &note.avatar_url,
                        user_name: &note.user_name,
                        note_info: &note.note_info,
                        text: &note.text,
                        reactions: note.reactions.clone(),
                        branch_fragments: note.branch_fragments.clone()
                    }
                }
            }
        }
    }
}
