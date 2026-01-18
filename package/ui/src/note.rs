use {crate::NoteSnippets, dioxus::prelude::*, schema::Note as NoteStruct};

#[component]
pub fn Note(note: NoteStruct) -> Element {
    let snippets = note.snippets;

    match note.id_ {
        Some(id) => rsx! {
            p { "Note: {id}" }

            NoteSnippets { snippets }
        },
        _ => rsx! {
            NoteSnippets { snippets }
        },
    }
}
