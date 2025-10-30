use {
    dioxus::{logger::tracing, prelude::*},
    schema::Note,
};

const NOTE_EDITOR_CSS: Asset = asset!("/assets/styling/note_editor.css");

#[derive(Clone, PartialEq, Props)]
pub struct NoteEditorProps {
    handle_edited: Option<Callback<Note>>,
}

#[component]
pub fn NoteEditor(props: NoteEditorProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NOTE_EDITOR_CSS }

        form {
            id: "note-editor",
            onsubmit: move |event| async move {
                event.prevent_default();

                let values = event.data().values();
                let Some(form_value) = values.get("content") else {
                    return;
                };
                let content = form_value.as_value();

                match backend::save_note(content).await {
                    Ok(note) => {
                        props.handle_edited.unwrap_or_default().call(note);
                    },
                    Err(error) => {
                        tracing::error!("{error:?}");
                    }
                };
            },

            h4 { "Note Editor" }

            textarea {
                id: "note-editor-input",
                name: "content"
            }

            input {
                id: "note-editor-save-button",
                r#type: "submit"
            }
        }
    }
}
