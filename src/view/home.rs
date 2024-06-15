use std::ops::Deref;

use dioxus::prelude::*;

use crate::global_state::get_app_model;

#[derive(Clone, PartialEq, Eq, Props)]
pub struct NoteProps {
    #[props(into)]
    avatar_url: String,

    #[props(into)]
    user_name: String,

    #[props(into)]
    note_info: String,

    #[props(into)]
    text: String,
}

#[derive(Clone, PartialEq, Eq, Props)]
pub struct ColumnProps {
    notes: Signal<Vec<NoteProps>>,
}

#[component]
pub fn Home() -> Element {
    let mut notes = use_signal(|| Vec::<NoteProps>::new());

    spawn(async move {
        let mut rx = get_app_model()
            .read()
            .await
            .merged_timeline
            .write()
            .await
            .make_column_receiver();

        while let Some(model_notes) = rx.recv().await {
            *notes.write() = model_notes
                .into_iter()
                .map(|x| NoteProps {
                    avatar_url: x.user.avatar_url,
                    user_name: x.user.name.unwrap_or(x.user.username),
                    note_info: format!("{:?} {:?}", x.visibility, x.local_only),
                    text: x.text.unwrap_or("".to_owned()),
                })
                .collect();
        }
    });

    rsx! {
        Column {
            notes: notes
        }
    }
}

#[component]
pub fn Column(props: ColumnProps) -> Element {
    rsx! {
        div {
            for note in props.notes.read().deref() {
                article {
                    Note {
                        avatar_url: &note.avatar_url,
                        user_name: &note.user_name,
                        note_info: &note.note_info,
                        text: &note.text
                    }
                }
            }
        }
    }
}

#[component]
pub fn Note(props: NoteProps) -> Element {
    rsx!(
        div{
            class: "note",
            div{
                class: "avatar",
                img {  src: "{props.avatar_url}" }
            }
            div{
                class: "header",
            div {
                class: "user-name",
                    "{props.user_name}"
                }
                div {
                    class: "note-info",
                    "{props.note_info}"
                }
            }
            div {
                class: "body",
                "{props.text}"
            }
        }
    )
}
