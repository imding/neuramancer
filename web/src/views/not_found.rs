use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    println!("Page not found: {segments:?}");
    rsx! {
        div {
            id: "not-found",

            h1 { "Page Not Found" }
        }
    }
}
