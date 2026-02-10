use dioxus::prelude::*;

/// Demo component — styling handled by #demo rules in tailwind.css @layer components
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { id: "demo" }
    }
}
