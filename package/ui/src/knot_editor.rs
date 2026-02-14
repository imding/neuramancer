use {
    dioxus::{logger::tracing, prelude::*},
    schema::Knot,
};

#[derive(Clone, PartialEq, Props)]
pub struct KnotEditorProps {
    handle_created: Option<Callback<Knot>>,
    handle_cancel: Option<Callback<()>>,
}

const INPUT_CLASS: &str = "p-2 border border-[#444] bg-[#2a2e3a] text-white rounded focus:outline-none focus:border-[#6d85c6] disabled:opacity-60 disabled:cursor-not-allowed";

#[component]
pub fn KnotEditor(props: KnotEditorProps) -> Element {
    let mut label = use_signal(String::new);
    let mut intent = use_signal(String::new);
    let mut note_ids = use_signal(Vec::<String>::new);
    let mut knot_ids = use_signal(Vec::<String>::new);
    let mut is_saving = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    let is_valid = use_memo(move || {
        let label_valid = !label().trim().is_empty();
        let intent_valid = !intent().trim().is_empty();
        let has_ids = note_ids().iter().any(|id| !id.trim().is_empty()) ||
            knot_ids().iter().any(|id| !id.trim().is_empty());

        label_valid && intent_valid && has_ids
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
        if index < current.len() {
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
        div { class: "w-[480px] bg-[#1e222d] p-5 rounded-[10px] grid gap-4 [#graph-editor_&]:w-full [#graph-editor_&]:p-0 [#graph-editor_&]:bg-transparent [#graph-editor_&]:rounded-none",

            h4 { class: "mb-[15px] text-white", "Create New Knot" }

            if let Some(error) = error_message() {
                div { class: "p-3 bg-[#dc3545] text-white rounded text-sm", "{error}" }
            }

            div { class: "grid gap-2",
                label { class: "text-white font-medium text-sm", r#for: "label-input", "Label" }
                input {
                    class: INPUT_CLASS,
                    id: "label-input",
                    value: label(),
                    oninput: move |event| label.set(event.value()),
                }
                label { class: "text-white font-medium text-sm", r#for: "intent-input", "Intent" }
                textarea {
                    class: "{INPUT_CLASS} min-h-[80px] resize-y font-[inherit]",
                    id: "intent-input",
                    placeholder: "Describe the purpose or theme of this knot...",
                    value: intent(),
                    oninput: move |event| intent.set(event.value()),
                    disabled: is_saving(),
                }
            }

            div { class: "grid gap-2",
                label { class: "text-white font-medium text-sm", "Note IDs:" }
                if note_ids().is_empty() {
                    p { class: "text-[#888] italic text-sm my-2",
                        "No note IDs added. Click 'Add Note' to include notes in this knot."
                    }
                }
                for (index , note_id) in note_ids().iter().enumerate() {
                    div { class: "grid grid-cols-[1fr_auto] gap-2 items-center", key: "note-id-{index}",
                        input {
                            class: INPUT_CLASS,
                            r#type: "text",
                            placeholder: "Enter note ID...",
                            value: note_id.clone(),
                            oninput: {
                                move |event: Event<FormData>| update_note_id(index, event.value())
                            },
                            disabled: is_saving(),
                        }
                        button {
                            r#type: "button",
                            class: "py-1.5 px-3 bg-[#dc3545] text-white border-none rounded cursor-pointer text-xs transition-colors duration-200 hover:not-disabled:bg-[#c82333] disabled:opacity-60 disabled:cursor-not-allowed",
                            onclick: {
                                move |_| remove_note_field(index)
                            },
                            disabled: is_saving(),
                            "Remove"
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "py-2 px-4 bg-[#28a745] text-white border-none rounded cursor-pointer text-sm transition-colors duration-200 justify-self-start hover:not-disabled:bg-[#218838] disabled:opacity-60 disabled:cursor-not-allowed",
                    onclick: add_note_field,
                    disabled: is_saving(),
                    "Add Note"
                }
            }

            div { class: "grid gap-2",
                label { class: "text-white font-medium text-sm", "Knot IDs:" }
                if knot_ids().is_empty() {
                    p { class: "text-[#888] italic text-sm my-2",
                        "No knot IDs added. Click 'Add Knot' to include other knots in this knot."
                    }
                } else {
                    for (index , knot_id) in knot_ids().iter().enumerate() {
                        div { class: "grid grid-cols-[1fr_auto] gap-2 items-center", key: "knot-{index}",
                            input {
                                class: INPUT_CLASS,
                                r#type: "text",
                                placeholder: "Enter knot ID...",
                                value: knot_id.clone(),
                                oninput: {
                                    move |event: Event<FormData>| update_knot_id(index, event.value())
                                },
                                disabled: is_saving(),
                            }
                            button {
                                r#type: "button",
                                class: "py-1.5 px-3 bg-[#dc3545] text-white border-none rounded cursor-pointer text-xs transition-colors duration-200 hover:not-disabled:bg-[#c82333] disabled:opacity-60 disabled:cursor-not-allowed",
                                onclick: {
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
                    class: "py-2 px-4 bg-[#28a745] text-white border-none rounded cursor-pointer text-sm transition-colors duration-200 justify-self-start hover:not-disabled:bg-[#218838] disabled:opacity-60 disabled:cursor-not-allowed",
                    onclick: add_knot_field,
                    disabled: is_saving(),
                    "Add Knot"
                }
            }

            div { class: "grid grid-cols-2 gap-3 mt-4",
                button {
                    r#type: "button",
                    class: "py-2.5 px-4 bg-[#6c757d] text-white border-none rounded cursor-pointer transition-colors duration-200 hover:not-disabled:bg-[#5a6268] disabled:opacity-60 disabled:cursor-not-allowed",
                    onclick: handle_cancel,
                    disabled: is_saving(),
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "py-2.5 px-4 bg-[#6d85c6] text-white border-none rounded cursor-pointer transition-colors duration-200 hover:not-disabled:bg-[#5a73b3] disabled:opacity-60 disabled:cursor-not-allowed",
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

                            tracing::debug!(
                                "Creating knot with: label={}, intent={}, note_ids={:?}, knot_ids={:?}",
                                label(), intent(), filtered_note_ids, filtered_knot_ids
                            );
                            match backend::create_knot(label(), intent(), filtered_note_ids, filtered_knot_ids)
                                .await
                            {
                                Ok(knot) => {
                                    props.handle_created.unwrap_or_default().call(knot);
                                }
                                Err(error) => {
                                    tracing::error!("{error}");
                                    error_message.set(Some(format!("Failed to create knot: {error}")));
                                }
                            }
                            is_saving.set(false);
                        }
                    },
                    disabled: !is_valid() || is_saving(),

                    if is_saving() {
                        "Saving..."
                    } else {
                        "Save"
                    }
                }
            }
        }
    }
}
