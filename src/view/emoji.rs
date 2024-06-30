use dioxus::prelude::*;
use tracing::{debug, error};

use crate::{common_types::Host, global_state::get_decomposer};

#[derive(Clone, PartialEq, Eq, Props)]
pub struct EmojiProp {
    pub host: Host,
    pub name: String,
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
