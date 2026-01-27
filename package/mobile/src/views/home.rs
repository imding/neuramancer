use {
    dioxus::prelude::*,
    ui::{Echo, Hero},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
    }
}
