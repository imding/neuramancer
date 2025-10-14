use {
    dioxus::prelude::*,
    ui::{Hero, TextNoteEditor},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        TextNoteEditor {}
    }
}
