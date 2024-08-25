use dioxus::prelude::*;

use super::*;
use crate::{common_types::Host, global_state::get_decomposer};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct RenoteInfo {
    #[props(into)]
    pub avatar_url: String,

    #[props(into)]
    pub user_name: String,
}

#[derive(Clone, PartialEq, Eq, Props)]
pub struct NoteProps {
    #[props(into)]
    pub original_host: Host,

    #[props(into)]
    pub uri: String,

    #[props(into)]
    pub avatar_url: String,

    #[props(into)]
    pub user_name: String,

    #[props(into)]
    pub note_info: String,

    #[props(into)]
    pub text: String,

    #[props(into)]
    pub file_thumbnails: Vec<String>,

    #[props(into)]
    pub reactions: Vec<(String, i64)>,

    #[props(into)]
    pub branch_fragments: Vec<BranchFragment>,

    #[props(into)]
    pub renote: Option<RenoteInfo>,
}

#[derive(Clone, PartialEq, Eq, Props)]
pub struct BranchFragment {
    pub color: String,
    pub view: BranchFragmentView,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BranchFragmentView {
    None,
    Top,
    Full,
    Skip,
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

    let branch_line_scale = 1000 / 25;

    let branches = props.branch_fragments.iter().map(|x| {
        let top_line = rsx! {
            div { class: "svg-container",
                svg {
                    style: "transform: scaleY({branch_line_scale})",
                    view_box: "0 0 100 1000",
                    width: 100,
                    height: 100,
                    preserve_aspect_ratio: "none",
                    line {
                        x1: "50",
                        x2: "50",
                        y1: "500",
                        y2: "1000",
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
                    style: "transform: scaleY({branch_line_scale})",
                    view_box: "0 0 100 1000",
                    width: 100,
                    height: 100,
                    preserve_aspect_ratio: "none",
                    line {
                        x1: "50",
                        x2: "50",
                        y1: "0",
                        y2: "1000",
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
        div { class: "note-row",
            div { class: "branches", {branches} }
            article { class: "note",

                if let Some(renote) = props.renote {
                    div { class: "renote-header",
                        div {
                            img { src: "{renote.avatar_url}" }
                        }
                        div {
                            span { {renote.user_name} }
                        }
                    }
                }
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
                div { class: "files",
                    for x in props.file_thumbnails {
                        img { class: "file_thumbnail", src: x }
                    }
                }
                div { class: "reactions",
                    for (r , n) in props.reactions {
                        Reaction { name: r, count: n }
                    }
                }
            }
        }
    }
}
