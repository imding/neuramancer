use {
    backend::NewNote,
    dioxus::{html::FormValue, logger::tracing, prelude::*},
};

const NOTE_EDITOR_CSS: Asset = asset!("/assets/styling/note_editor.css");

#[derive(Clone, PartialEq, Props)]
pub struct NoteEditorProps {
    handle_edited: Option<Callback<NewNote>>,
}

#[component]
pub fn NoteEditor(props: NoteEditorProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NOTE_EDITOR_CSS }

        form {
            id: "note-editor",
            onsubmit: move |event| async move {
                event.prevent_default();
                let Some(form_value) = event.data().get_first("content") else {
                    return;
                };
                let content = match form_value {
                    FormValue::Text(text) => text,
                    _ => return,
                };
                match backend::save_note(content).await {
                    Ok(note_created) => {
                        props.handle_edited.unwrap_or_default().call(note_created);
                    }
                    Err(error) => {
                        tracing::error!("{error:?}");
                    }
                };
            },

            h4 { "Note Editor" }

            textarea { id: "note-editor-input", name: "content" }

            input { id: "note-editor-save-button", r#type: "submit" }
        }
    }
}
