use {
    dioxus::{logger::tracing, prelude::*},
    schema::Knot,
};

const KNOT_EDITOR_CSS: Asset = asset!("/assets/styling/knot_editor.css");

#[derive(Clone, PartialEq, Props)]
pub struct KnotEditorProps {
    handle_created: Option<Callback<Knot>>,
    handle_cancel: Option<Callback<()>>,
}

#[component]
pub fn KnotEditor(props: KnotEditorProps) -> Element {
    let mut intent = use_signal(|| String::new());
    let mut note_ids = use_signal(|| vec![String::new()]);
    let mut knot_ids = use_signal(|| Vec::<String>::new());
    let mut is_saving = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    let is_valid = use_memo(move || {
        let intent_valid = !intent().trim().is_empty();
        let has_ids = note_ids().iter().any(|id| !id.trim().is_empty()) 
                     || knot_ids().iter().any(|id| !id.trim().is_empty());
        intent_valid && has_ids
    });

    let add_note_field = move |_| {
        let mut current = note_ids();
        current.push(String::new());
        note_ids.set(current);
    };

    let add_knot_field = move |_| {
        let mut current = knot_ids();
        current.push(String::new());
        knot_ids.set(current);
    };

    let mut update_note_id = move |index: usize, value: String| {
        let mut current = note_ids();
        if index < current.len() {
            current[index] = value;
            note_ids.set(current);
        }
    };

    let mut update_knot_id = move |index: usize, value: String| {
        let mut current = knot_ids();
        if index < current.len() {
            current[index] = value;
            knot_ids.set(current);
        }
    };

    let mut remove_note_field = move |index: usize| {
        let mut current = note_ids();
        if current.len() > 1 && index < current.len() {
            current.remove(index);
            note_ids.set(current);
        }
    };

    let mut remove_knot_field = move |index: usize| {
        let mut current = knot_ids();
        if index < current.len() {
            current.remove(index);
            knot_ids.set(current);
        }
    };



    let handle_cancel = move |_| {
        props.handle_cancel.unwrap_or_default().call(());
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KNOT_EDITOR_CSS }

        div {
            id: "knot-editor",

            h4 { "Create New Knot" }

            if let Some(error) = error_message() {
                div {
                    class: "error-message",
                    "{error}"
                }
            }

            div {
                class: "form-group",
                label { r#for: "intent-input", "Intent (required):" }
                textarea {
                    id: "intent-input",
                    placeholder: "Describe the purpose or theme of this knot...",
                    value: intent(),
                    oninput: move |event: FormEvent| intent.set(event.value()),
                    disabled: is_saving()
                }
            }

            div {
                class: "form-group",
                label { "Note IDs:" }
                for (index, note_id) in note_ids().iter().enumerate() {
                    div {
                        class: "input-row",
                        key: "note-{index}",
                        input {
                            r#type: "text",
                            placeholder: "Enter note ID...",
                            value: note_id.clone(),
                            oninput: {
                                let index = index;
                                move |event: Event<FormData>| update_note_id(index, event.value())
                            },
                            disabled: is_saving()
                        }
                        if note_ids().len() > 1 {
                            button {
                                r#type: "button",
                                class: "remove-button",
                                onclick: {
                                    let index = index;
                                    move |_| remove_note_field(index)
                                },
                                disabled: is_saving(),
                                "Remove"
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "add-button",
                    onclick: add_note_field,
                    disabled: is_saving(),
                    "Add Note"
                }
            }

            div {
                class: "form-group",
                label { "Knot IDs:" }
                if knot_ids().is_empty() {
                    p { class: "empty-state", "No knot IDs added yet" }
                } else {
                    for (index, knot_id) in knot_ids().iter().enumerate() {
                        div {
                            class: "input-row",
                            key: "knot-{index}",
                            input {
                                r#type: "text",
                                placeholder: "Enter knot ID...",
                                value: knot_id.clone(),
                                oninput: {
                                    let index = index;
                                    move |event: Event<FormData>| update_knot_id(index, event.value())
                                },
                                disabled: is_saving()
                            }
                            button {
                                r#type: "button",
                                class: "remove-button",
                                onclick: {
                                    let index = index;
                                    move |_| remove_knot_field(index)
                                },
                                disabled: is_saving(),
                                "Remove"
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "add-button",
                    onclick: add_knot_field,
                    disabled: is_saving(),
                    "Add Knot"
                }
            }

            div {
                class: "button-group",
                button {
                    r#type: "button",
                    class: "cancel-button",
                    onclick: handle_cancel,
                    disabled: is_saving(),
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "save-button",
                    onclick: move |_| {
                        async move {
                            if !is_valid() {
                                return;
                            }

                            is_saving.set(true);
                            error_message.set(None);

                            let filtered_note_ids: Vec<String> = note_ids()
                                .into_iter()
                                .filter(|id| !id.trim().is_empty())
                                .collect();

                            let filtered_knot_ids: Vec<String> = knot_ids()
                                .into_iter()
                                .filter(|id| !id.trim().is_empty())
                                .collect();

                            match backend::create_knot(filtered_knot_ids, filtered_note_ids, intent()).await {
                                Ok(knot) => {
                                    tracing::debug!("Successfully created knot: {:?}", knot);
                                    props.handle_created.unwrap_or_default().call(knot);
                                }
                                Err(error) => {
                                    tracing::error!("Failed to create knot: {:?}", error);
                                    error_message.set(Some(format!("Failed to create knot: {}", error)));
                                }
                            }

                            is_saving.set(false);
                        }
                    },
                    disabled: !is_valid() || is_saving(),
                    if is_saving() { "Saving..." } else { "Save" }
                }
            }
        }
    }
}
