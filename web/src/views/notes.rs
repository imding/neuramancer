use {dioxus::prelude::*, ui::NoteList};

#[component]
pub fn Notes() -> Element {
    rsx! {
        div {
            id: "notes",

            h1 { "Notes" }

            NoteList {}
        }
    }
}
