use {
    dioxus::prelude::*,
    ui::{Echo, Hero, TextNoteEditor},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        TextNoteEditor {}
        Echo {}
    }
}
