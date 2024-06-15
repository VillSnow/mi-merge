use std::ops::Deref;

use dioxus::prelude::*;
use tracing::{debug, error};

use crate::{
    common_types::Host,
    global_state::{get_app_model, get_decomposer},
};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct NoteProps {
    #[props(into)]
    host: Host,

    #[props(into)]
    uri: String,

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

#[derive(Clone, PartialEq, Eq, Props)]
pub struct EmojiProp {
    host: Host,
    name: String,
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
                    host: x.host,
                    uri: x.uri,
                    avatar_url: x.note.user.avatar_url,
                    user_name: x.note.user.name.unwrap_or(x.note.user.username),
                    note_info: format!(
                        "{} {:?} {:?}",
                        x.note.created_at, x.note.visibility, x.note.local_only
                    ),
                    text: x.note.text.unwrap_or("".to_owned()),
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
                    key: "{note.uri}",
                    Note {
                        host: note.host.clone(),
                        uri: &note.uri,
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
    let decomposed = get_decomposer().decompose(&props.text);
    let body = decomposed.into_iter().map(|x| match x {
        crate::mfm::DecomposedTextItem::Text(x) => rsx!("{x}"),
        crate::mfm::DecomposedTextItem::Emoji(x) => rsx!(Emoji {
            host: props.host.clone(),
            name: x
        }),
    });
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
                span { {body} }
            }
        }
    )
}

#[component]
pub fn Emoji(props: EmojiProp) -> Element {
    debug!("rendering emoji {}", props.name);

    async fn f(props: EmojiProp) -> Option<String> {
        let emoji = get_decomposer()
            .fetch_emoji(&props.host, &props.name)
            .await
            .map_err(|e| error!("failed to fetch emoji url: {e}"))
            .ok()?;
        Some(emoji.url)
    }
    let url = use_resource({
        let props = props.clone();
        move || f(props.clone())
    });

    let url = url.read();
    if let Some(url) = url.as_ref().and_then(|x| x.as_ref()) {
        rsx!(img {
            class: "emoji",
            src: url.as_str()
        })
    } else {
        rsx!(span { ":{props.name}:" })
    }
}
