use dioxus::{logger::tracing, prelude::*};

const TEXT_NOTE_EDITOR_CSS: Asset = asset!("/assets/styling/text_note_editor.css");

#[component]
pub fn TextNoteEditor() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TEXT_NOTE_EDITOR_CSS }

        form {
            id: "text-note-editor",
            onsubmit: move |event| async move {
                event.prevent_default();

                let values = event.data().values();
                let Some(form_value) = values.get("content") else {
                    return;
                };
                let content = form_value.as_value();
                let Ok(response) = backend::save_text_note(content).await else {
                    return tracing::error!("Failed to save text note");
                };

                tracing::debug!("{response:?}");
            },

            h4 { "Text Note Editor" }

            textarea {
                id: "text-note-editor-input",
                name: "content"
            }

            input {
                id: "text-note-editor-save-button",
                r#type: "submit"
            }
        }
    }
}
