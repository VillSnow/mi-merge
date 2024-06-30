use dioxus::prelude::*;
use fancy_regex::Regex;
use tracing::debug;

use super::*;
use crate::common_types::Host;

#[derive(Clone, PartialEq, Eq, Props)]
pub struct ReactionProp {
    pub name: String,
    pub count: i64,
}

#[component]
pub fn Reaction(props: ReactionProp) -> Element {
    debug!("rendering reaction {}", props.name);

    let re = Regex::new("^:(.*)@(.*):$").unwrap();
    match re.captures(&props.name).expect("regex error") {
        Some(captures) => rsx! {
            div { class: "reaction-button",
                Emoji {
                    host: Host::from(captures.get(2).unwrap().as_str().to_owned()),
                    name: captures.get(1).unwrap().as_str()
                }
                span { "{props.count}" }
            }
        },

        None => rsx! {
            div { class: "reaction-button",
                span { "{props.name}" }
                span { "{props.count}" }
            }
        },
    }
}
