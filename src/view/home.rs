use std::{collections::HashSet, ops::Deref};

use dioxus::prelude::*;
use fancy_regex::Regex;
use tracing::{debug, error};

use crate::{
    common_types::{BranchKey, Host},
    global_state::{get_app_model, get_decomposer},
};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct NoteProps {
    #[props(into)]
    original_host: Host,

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

    #[props(into)]
    reactions: Vec<(String, i64)>,

    #[props(into)]
    branch_fragments: Vec<BranchFragment>,
}

#[derive(Clone, PartialEq, Eq, Props)]
pub struct BranchFragment {
    color: String,
    view: BranchFragmentView,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BranchFragmentView {
    None,
    Top,
    Full,
    Skip,
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

#[derive(Clone, PartialEq, Eq, Props)]
pub struct ReactionProp {
    name: String,
    count: i64,
}

#[component]
pub fn Home() -> Element {
    let mut notes = use_signal(|| Vec::<NoteProps>::new());

    spawn(async move {
        let branches = get_app_model().read().await.branches.clone();
        let mut rx = get_app_model()
            .read()
            .await
            .merged_timeline
            .write()
            .await
            .make_column_receiver();

        while let Some(model_notes) = rx.recv().await {
            let mut notes_prop = Vec::new();
            let mut branch_trace = HashSet::<BranchKey>::new();

            for x in model_notes {
                notes_prop.push(NoteProps {
                    original_host: x.original_host,
                    uri: x.uri,
                    avatar_url: x.mi_note.user.avatar_url,
                    user_name: x.mi_note.user.name.unwrap_or(x.mi_note.user.username),
                    note_info: format!(
                        "{} {:?} {:?}",
                        x.mi_note.created_at, x.mi_note.visibility, x.mi_note.local_only
                    ),
                    text: x.mi_note.text.unwrap_or("".to_owned()),
                    reactions: x.reactions.clone(),
                    branch_fragments: branches
                        .iter()
                        .enumerate()
                        .map(|(i, y)| BranchFragment {
                            color: make_color(i),
                            view: if branch_trace.contains(y) {
                                if x.branches.contains(y) {
                                    BranchFragmentView::Full
                                } else {
                                    BranchFragmentView::Skip
                                }
                            } else if x.branches.contains(&y) {
                                BranchFragmentView::Top
                            } else {
                                BranchFragmentView::None
                            },
                        })
                        .collect(),
                });

                branch_trace.extend(x.branches);
            }

            *notes.write() = notes_prop;
        }
    });

    rsx! {
        Column { notes }
    }
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

#[component]
pub fn Note(props: NoteProps) -> Element {
    let decomposed = get_decomposer().decompose(&props.user_name);
    let username = decomposed.into_iter().map(|x| match x {
        crate::mfm::DecomposedTextItem::Text(x) => rsx! { "{x}" },
        crate::mfm::DecomposedTextItem::Emoji(x) => rsx! {
            Emoji { host: props.original_host.clone(), name: x }
        },
    });

    let decomposed = get_decomposer().decompose(&props.text);
    let body = decomposed.into_iter().map(|x| match x {
        crate::mfm::DecomposedTextItem::Text(x) => rsx! { "{x}" },
        crate::mfm::DecomposedTextItem::Emoji(x) => rsx! {
            Emoji { host: props.original_host.clone(), name: x }
        },
    });
    let branches = props.branch_fragments.iter().map(|x| {
        let top_line = rsx! {
            div { class: "svg-container",
                svg {
                    class: "branch-line",
                    view_box: "0 0 100 100",
                    width: 100,
                    height: 100,
                    preserve_aspect_ratio: "none",
                    line {
                        x1: "50",
                        x2: "50",
                        y1: "50",
                        y2: "100",
                        fill: "none",
                        stroke: "{x.color}",
                        stroke_width: "20"
                    }
                }
            }
        };
        let full_line = rsx! {
            div { class: "svg-container",
                svg {
                    class: "branch-line",
                    view_box: "0 0 100 100",
                    width: 100,
                    height: 100,
                    preserve_aspect_ratio: "none",
                    line {
                        x1: "50",
                        x2: "50",
                        y1: "0",
                        y2: "100",
                        fill: "none",
                        stroke: "{x.color}",
                        stroke_width: "20"
                    }
                }
            }
        };
        let dot = rsx! {
            div { class: "svg-container",
                svg {
                    class: "branch-dot",
                    view_box: "0 0 100 100",
                    width: 100,
                    height: 200,
                    circle {
                        cx: "50",
                        cy: "50",
                        r: "20",
                        fill: "#ccc",
                        stroke: "{x.color}",
                        stroke_width: "15"
                    }
                }
            }
        };
        match x.view {
            BranchFragmentView::None => rsx! {
                div { class: "branch-fragment" }
            },
            BranchFragmentView::Top => rsx! {
                div { class: "branch-fragment",
                    {top_line},
                    {dot}
                }
            },
            BranchFragmentView::Full => rsx! {
                div { class: "branch-fragment",
                    {full_line},
                    {dot}
                }
            },
            BranchFragmentView::Skip => rsx! {
                div { class: "branch-fragment", {full_line} }
            },
        }
    });

    rsx! {
        div { class: "note",
            div { class: "branches", {branches} }
            div { class: "avatar",
                img { src: "{props.avatar_url}" }
            }
            div { class: "header",
                div { class: "user-name",
                    span { {username} }
                }
                div { class: "note-info", "{props.note_info}" }
            }
            div { class: "body",
                span { {body} }
            }
            div { class: "reactions",
                for (r , n) in props.reactions {
                    Reaction { name: r, count: n }
                }
            }
        }
    }
}

#[component]
pub fn Emoji(props: EmojiProp) -> Element {
    debug!("rendering emoji {}", props.name);

    async fn f(props: EmojiProp) -> Option<String> {
        let emoji = get_decomposer()
            .fetch_emoji(&props.host, &props.name)
            .await
            .map_err(|e| error!("failed to fetch emoji url: {e:?}"))
            .ok()?;
        Some(emoji.url)
    }
    let url = use_resource({
        let props = props.clone();
        move || f(props.clone())
    });

    let url = url.read();
    if let Some(url) = url.as_ref().and_then(|x| x.as_ref()) {
        rsx! {
            img { class: "emoji", src: url.as_str() }
        }
    } else {
        rsx! {
            span { ":{props.name}:" }
        }
    }
}

#[component]
pub fn Reaction(props: ReactionProp) -> Element {
    debug!("rendering reaction {}", props.name);

    let re = Regex::new("^:(.*)@(.*):$").unwrap();
    match re.captures(&props.name).expect("regex error") {
        Some(captures) => rsx! {
            div {
                class: "reaction-button",
                Emoji {
                    host: Host::from(captures.get(2).unwrap().as_str().to_owned()),
                    name: captures.get(1).unwrap().as_str()
                }
                span { "{props.count}" }
            }
        },

        None => rsx! {
            div {
                class: "reaction-button",
                span { "{props.name}" }
                span { "{props.count}" }
            }
        },
    }
}

fn make_color(n: usize) -> String {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;

    let h = 360.0 / (1.0 + phi) * n as f64;
    let h = h % 360.0;
    let s = 1.0;
    let v = 1.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h {
        h if h >= 0.0 && h < 60.0 => (c, x, 0.0),
        h if h >= 60.0 && h < 120.0 => (x, c, 0.0),
        h if h >= 120.0 && h < 180.0 => (0.0, c, x),
        h if h >= 180.0 && h < 240.0 => (0.0, x, c),
        h if h >= 240.0 && h < 300.0 => (x, 0.0, c),
        h if h >= 300.0 && h < 360.0 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r = ((r + m) * 256.0).round().min(255.0) as u8;
    let g = ((g + m) * 256.0).round().min(255.0) as u8;
    let b = ((b + m) * 256.0).round().min(255.0) as u8;

    format!("#{r:02x}{g:02x}{b:02x}")
}
