use {
    dioxus::prelude::*,
    ui::{Demo, Hero},
};

const HOME_CSS: Asset = asset!("/assets/home.css");

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: HOME_CSS }

        div {
            id: "home-page",

            Hero {}
            Demo {}
        }
    }
}
