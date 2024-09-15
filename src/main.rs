mod app_model;
mod cached_req;
mod common_types;
mod emoji_service;
mod global_state;
mod merged_timeline;
mod mfm;
mod mi_models;
mod server_cxn;
mod server_note_repo;
mod view;
mod ws_msg_router;
mod ws_poller;

use dioxus::{desktop::use_window, prelude::*};

use global_state::get_app_model;

use tracing::Level;

use crate::view::Home;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

fn main() {
    dotenv::dotenv().ok();
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_window().window.set_always_on_top(false);

    spawn(async {
        get_app_model()
            .write()
            .await
            .connect_all()
            .await
            .expect("TODO: connect error");
    });

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        Router::<Route> {}
    }
}
