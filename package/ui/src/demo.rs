use dioxus::prelude::*;

const DEMO_CSS: Asset = asset!("/assets/styling/demo.css");

#[component]
pub fn Demo() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: DEMO_CSS }

        div { id: "demo" }
    }
}
