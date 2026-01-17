use {
    crate::use_notes_store,
    dioxus::{html::FormValue, prelude::*},
};

const NOTE_EDITOR_CSS: Asset = asset!("/assets/styling/note_editor.css");

#[derive(Clone, PartialEq, Props)]
pub struct NoteEditorProps {
    /// Optional callback for callers that still want to observe the edit.
    ///
    /// Note: optimistic creation is handled by `NotesStore`. This callback is best-effort
    /// (it will be invoked after the server call succeeds only if wired up elsewhere).
    handle_edited: Option<Callback<backend::NewNote>>,
}

#[component]
pub fn NoteEditor(props: NoteEditorProps) -> Element {
    let store = use_notes_store();

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

                // Optimistic path: update local cache immediately, then hydrate/rollback based on server response.
                store.read().create_note_optimistic(content);

                // Legacy callback: creation is now handled by the store. If a caller needs server-created IDs,
                // that should be surfaced via store state instead of this callback.
                let _ = props.handle_edited;
            },

            h4 { "Note Editor" }

            textarea { id: "note-editor-input", name: "content" }

            input { id: "note-editor-save-button", r#type: "submit" }
        }
    }
}
