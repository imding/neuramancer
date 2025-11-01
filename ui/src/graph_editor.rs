use {
    crate::KnotEditor,
    dioxus::{logger::tracing, prelude::*},
};

const GRAPH_EDITOR_CSS: Asset = asset!("/assets/styling/graph_editor.css");

#[derive(Clone, PartialEq, Props)]
pub struct GraphEditorProps {
    handle_updated: Option<Callback<()>>,
}

#[component]
pub fn GraphEditor(props: GraphEditorProps) -> Element {
    let mut active_tab = use_signal(|| "notes");
    let mut show_knot_editor = use_signal(|| false);
    let mut notes = use_resource(move || backend::read_notes());
    let mut knots = use_resource(move || backend::read_knots());

    let mut refresh_data = move || {
        notes.restart();
        knots.restart();
        props.handle_updated.unwrap_or_default().call(());
    };

    rsx! {
        document::Link { rel: "stylesheet", href: GRAPH_EDITOR_CSS }

        button {
            id: "graph-editor-trigger",
            popovertarget: "graph-editor-wrapper",

            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "512",
                height: "512",
                view_box: "0 0 512 512",

                path {
                    fill: "currentColor",
                    d: "m102.53 26.063l90 345.75l289.22 23.25l-90.03-345.72zm-18.968 1.406c-30.44 11.894-55.62 53.07-49.687 75.28l3.25 11.813c.654-1.722 1.345-3.44 2.063-5.157C49.102 85.688 65.734 62.636 89.56 50.5l-6-23.03zM94.44 69.187c-16.66 10.016-29.916 28.1-38 47.437c-5.2 12.44-8 25.417-8.75 36.25v.03L112.56 388.5c.305-.572.593-1.148.907-1.72c10.585-19.223 27.804-37.623 51.06-48.405L94.438 69.187zM154 107.968l239.78 16.188l-1.28 18.625l-239.75-16.155L154 107.97zm46.03 34.407l5.657 8.875l14.188 22.313l39.03-15.25l7.595-2.938l3.97 7.094l16.28 29.124l4.313 7.72l-7.438 4.717c-10.267 6.524-17.392 12.284-21.75 16.782c-3.03 3.13-4.247 5.232-4.906 6.594c1.38.303 3.433.577 6.624.28c18.268-1.69 56.285-19.964 79-61.592l5.47-10.03l8.748 7.374l46 38.812l11.532 9.72l-13.844 6l-33.28 14.374c5.447 4.925 11.436 5.916 18.436 5.406c9.95-.724 21.427-6.07 29.125-11.063l10.158 15.657c-9.41 6.1-22.867 12.934-37.938 14.03c-15.07 1.098-32.27-5.296-42.594-23.155l-5.25-9.095l9.625-4.156l30.44-13.157l-26.033-22c-25.716 40.294-62.68 59.168-87.843 61.5c-6.78.628-12.945.26-18.594-2.688c-5.65-2.95-9.984-10.6-9-17.406s4.838-12.4 10.688-18.44c4.385-4.526 10.612-9.367 17.875-14.436l-8.188-14.656L219.5 193.75l-7.156 2.78l-4.125-6.468L196 170.875c-6.308 7.158-9.485 14.528-9 21.406c.654 9.28 7.854 21.054 30.594 33.69l-9.094 16.343c-25.688-14.273-38.877-31.016-40.125-48.72c-1.248-17.703 9.393-33.013 23.5-44.562l8.156-6.655zm-5.968 118.188l239.782 16.156l-1.25 18.655l-239.78-16.188l1.25-18.625zm-24.75 96.25c-17.637 9.072-31.065 23.708-39.468 38.968c-4.49 8.153-7.307 16.452-8.72 23.876l11.626 42.156l1.688.157c-3.824-27.514 11.358-60.383 41.187-80.97zm26.22 34c-32.403 17.28-46.273 52.303-41.657 72.78l289.78 24.532c-5.298-7.743-8.625-17.827-8.592-28.313l-22.47-9.03l46.626-7.313l-13.69-13.064c5.552-6.838 13.54-12.915 24.47-17.53l-274.47-22.063z"
                }
            }
        }

        div {
            id: "graph-editor-wrapper",
            popover: "auto",

            if show_knot_editor() {
                KnotEditor {
                    handle_created: move |_knot| {
                        show_knot_editor.set(false);
                        active_tab.set("knots");
                        refresh_data();
                    },
                    handle_cancel: move |_| {
                        show_knot_editor.set(false);
                    }
                }
            } else {
                div {
                    id: "graph-editor",

                    h4 { "Graph Editor" }

                    div {
                        id: "tab-buttons",
                        button {
                            class: if active_tab() == "notes" { "active" } else { "" },
                            onclick: move |_| active_tab.set("notes"),
                            "Notes"
                        }
                        button {
                            class: if active_tab() == "knots" { "active" } else { "" },
                            onclick: move |_| active_tab.set("knots"),
                            "Knots"
                        }
                    }

                    div {
                        id: "tab-content",

                        if active_tab() == "notes" {
                            div {
                                id: "notes-tab",
                                match notes() {
                                    Some(Ok(notes_data)) => rsx! {
                                        if notes_data.is_empty() {
                                            p { class: "empty-state", "No notes found" }
                                        } else {
                                            for note in notes_data {
                                                div {
                                                    class: "note-item",
                                                    key: "note-{note.id_.clone().unwrap_or_default()}",

                                                    div {
                                                        class: "note-content",
                                                        p { "Note ID: {note.id_.clone().unwrap_or_default()}" }
                                                        p { "Snippets: {note.snippets.len()}" }
                                                    }

                                                    button {
                                                        class: "delete-button",
                                                        onclick: {
                                                            let note_id = note.id_.clone().unwrap_or_default();
                                                            move |_| {
                                                                let note_id = note_id.clone();
                                                                async move {
                                                                    match backend::delete_note(note_id.clone()).await {
                                                                        Ok(()) => {
                                                                            tracing::debug!("Successfully deleted note: {}", note_id);
                                                                            refresh_data();
                                                                        }
                                                                        Err(error) => {
                                                                            tracing::error!("Failed to delete note {}: {:?}", note_id, error);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        "Delete"
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Some(Err(error)) => rsx! {
                                        p { class: "error", "Error loading notes: {error}" }
                                    },
                                    None => rsx! {
                                        p { class: "loading", "Loading notes..." }
                                    }
                                }
                            }
                        } else {
                            div {
                                id: "knots-tab",

                                button {
                                    id: "create-knot-button",
                                    onclick: move |_| show_knot_editor.set(true),
                                    "Create New Knot"
                                }

                                match knots() {
                                    Some(Ok(knots_data)) => rsx! {
                                        if knots_data.is_empty() {
                                            p { class: "empty-state", "No knots found" }
                                        } else {
                                            for knot in knots_data {
                                                div {
                                                    class: "knot-item",
                                                    key: "knot-{knot.id_.clone().unwrap_or_default()}",

                                                    div {
                                                        class: "knot-content",
                                                        p { class: "knot-intent", "{knot.intent}" }
                                                        p { "Notes: {knot.notes.len()}, Knots: {knot.knots.len()}" }
                                                    }

                                                    button {
                                                        class: "delete-button",
                                                        onclick: {
                                                            let knot_id = knot.id_.clone().unwrap_or_default();
                                                            move |_| {
                                                                let knot_id = knot_id.clone();
                                                                async move {
                                                                    match backend::delete_knot(knot_id.clone(), false).await {
                                                                        Ok(()) => {
                                                                            tracing::debug!("Successfully deleted knot: {}", knot_id);
                                                                            refresh_data();
                                                                        }
                                                                        Err(error) => {
                                                                            tracing::error!("Failed to delete knot {}: {:?}", knot_id, error);
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        },
                                                        "Delete"
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Some(Err(error)) => rsx! {
                                        p { class: "error", "Error loading knots: {error}" }
                                    },
                                    None => rsx! {
                                        p { class: "loading", "Loading knots..." }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
