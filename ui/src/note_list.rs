use {crate::Note, dioxus::prelude::*};

#[component]
pub fn NoteList() -> Element {
    let notes = use_server_future(move || backend::read_notes())?;

    rsx! {
        div {
            id: "note-list",

            for note in notes().unwrap().unwrap() {
                Note { note }
            }
        }
    }
}
