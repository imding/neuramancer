use dioxus::prelude::*;

const NOTE_LIST_CSS: Asset = asset!("/assets/styling/note_list.css");

#[component]
pub fn NoteList() -> Element {
    let notes = use_server_future(move || backend::read_text_notes())?;

    rsx! {
        document::Link{ rel: "stylesheet", href: NOTE_LIST_CSS }

        div {
            id: "note-list",

            for note in notes().unwrap().unwrap() {
                div {
                    p { "ID: {note.id}" }
                    p { "Content: {note.content} "}
                    p { "Created at: {note.created_at} "}
                }
            }
        }
    }
}
