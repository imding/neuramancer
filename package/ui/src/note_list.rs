use {crate::Note, dioxus::prelude::*};

#[component]
pub fn NoteList() -> Element {
    let notes = use_resource(backend::read_notes);

    rsx! {
        div { id: "note-list",

            match notes() {
                Some(Ok(notes)) => rsx! {
                    for (index , note) in notes.iter().enumerate() {
                        Note { key: "note-{index}", note: note.clone() }
                    }
                },
                Some(Err(error)) => rsx! {
                    p { "Error: {error}" }
                },
                _ => rsx! {
                    p { "Loading..." }
                },
            }
        }
    }
}
