use dioxus::prelude::*;

use super::*;
use crate::{common_types::Host, global_state::get_decomposer};

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
    pub reactions: Vec<(String, i64)>,

    #[props(into)]
    pub branch_fragments: Vec<BranchFragment>,
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
