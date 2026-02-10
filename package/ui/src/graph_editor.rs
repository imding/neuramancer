use {
    crate::{KnotEditor, use_notes_store},
    dioxus::{logger::tracing, prelude::*},
    schema::Knot,
};

#[derive(Clone, PartialEq, Props)]
pub struct GraphEditorProps {
    handle_updated: Option<Callback<()>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Notes,
    Knots,
}

#[component]
pub fn GraphEditor(props: GraphEditorProps) -> Element {
    let mut active_tab = use_signal(|| Tab::Notes);

    rsx! {
        button {
            class: "bg-white border-none p-4 cursor-pointer rounded-full [&_svg]:w-full [&_svg]:h-full",
            popovertarget: "graph-editor",

            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "512",
                height: "512",
                view_box: "0 0 512 512",

                path {
                    fill: "currentColor",
                    d: "m102.53 26.063l90 345.75l289.22 23.25l-90.03-345.72zm-18.968 1.406c-30.44 11.894-55.62 53.07-49.687 75.28l3.25 11.813c.654-1.722 1.345-3.44 2.063-5.157C49.102 85.688 65.734 62.636 89.56 50.5l-6-23.03zM94.44 69.187c-16.66 10.016-29.916 28.1-38 47.437c-5.2 12.44-8 25.417-8.75 36.25v.03L112.56 388.5c.305-.572.593-1.148.907-1.72c10.585-19.223 27.804-37.623 51.06-48.405L94.438 69.187zM154 107.968l239.78 16.188l-1.28 18.625l-239.75-16.155L154 107.97zm46.03 34.407l5.657 8.875l14.188 22.313l39.03-15.25l7.595-2.938l3.97 7.094l16.28 29.124l4.313 7.72l-7.438 4.717c-10.267 6.524-17.392 12.284-21.75 16.782c-3.03 3.13-4.247 5.232-4.906 6.594c1.38.303 3.433.577 6.624.28c18.268-1.69 56.285-19.964 79-61.592l5.47-10.03l8.748 7.374l46 38.812l11.532 9.72l-13.844 6l-33.28 14.374c5.447 4.925 11.436 5.916 18.436 5.406c9.95-.724 21.427-6.07 29.125-11.063l10.158 15.657c-9.41 6.1-22.867 12.934-37.938 14.03c-15.07 1.098-32.27-5.296-42.594-23.155l-5.25-9.095l9.625-4.156l30.44-13.157l-26.033-22c-25.716 40.294-62.68 59.168-87.843 61.5c-6.78.628-12.945.26-18.594-2.688c-5.65-2.95-9.984-10.6-9-17.406s4.838-12.4 10.688-18.44c4.385-4.526 10.612-9.367 17.875-14.436l-8.188-14.656L219.5 193.75l-7.156 2.78l-4.125-6.468L196 170.875c-6.308 7.158-9.485 14.528-9 21.406c.654 9.28 7.854 21.054 30.594 33.69l-9.094 16.343c-25.688-14.273-38.877-31.016-40.125-48.72c-1.248-17.703 9.393-33.013 23.5-44.562l8.156-6.655zm-5.968 118.188l239.782 16.156l-1.25 18.655l-239.78-16.188l1.25-18.625zm-24.75 96.25c-17.637 9.072-31.065 23.708-39.468 38.968c-4.49 8.153-7.307 16.452-8.72 23.876l11.626 42.156l1.688.157c-3.824-27.514 11.358-60.383 41.187-80.97zm26.22 34c-32.403 17.28-46.273 52.303-41.657 72.78l289.78 24.532c-5.298-7.743-8.625-17.827-8.592-28.313l-22.47-9.03l46.626-7.313l-13.69-13.064c5.552-6.838 13.54-12.915 24.47-17.53l-274.47-22.063z",
                }
            }
        }

        div {
            id: "graph-editor",
            popover: "auto",
            class: "w-[480px] bg-[#1e222d] p-5 rounded-[10px] gap-4 border-none",

            h4 { class: "mb-[15px] text-white", "Graph Editor" }

            div { class: "grid grid-cols-2 gap-2 mb-4",
                button {
                    class: if *active_tab.read() == Tab::Notes {
                        "py-2 px-4 border border-[#6d85c6] bg-[#6d85c6] text-white rounded cursor-pointer transition-all duration-200"
                    } else {
                        "py-2 px-4 border border-[#444] bg-[#2a2e3a] text-[#ccc] rounded cursor-pointer transition-all duration-200 hover:bg-[#3a3e4a] hover:text-white"
                    },
                    onclick: move |_| active_tab.set(Tab::Notes),
                    "Notes"
                }
                button {
                    class: if *active_tab.read() == Tab::Knots {
                        "py-2 px-4 border border-[#6d85c6] bg-[#6d85c6] text-white rounded cursor-pointer transition-all duration-200"
                    } else {
                        "py-2 px-4 border border-[#444] bg-[#2a2e3a] text-[#ccc] rounded cursor-pointer transition-all duration-200 hover:bg-[#3a3e4a] hover:text-white"
                    },
                    onclick: move |_| active_tab.set(Tab::Knots),
                    "Knots"
                }
            }

            div { class: "min-h-[300px]",

                match *active_tab.read() {
                    Tab::Notes => rsx! {
                        NotesTab {}
                    },
                    Tab::Knots => rsx! {
                        KnotsTab { handle_knot_created: move |_| props.handle_updated.unwrap_or_default().call(()) }
                    },
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct NotesTabProps {}

#[component]
fn NotesTab() -> Element {
    let store = use_notes_store();
    let notes = store.read().state.read().items.clone();

    rsx! {
        match notes.is_empty() {
            true => rsx! {
                p { class: "text-[#888] text-center py-10 px-5 italic", "No notes found" }
            },
            _ => rsx! {
                for note in notes {
                    div { class: "grid grid-cols-[1fr_auto] grid-rows-[auto_auto_auto] gap-x-3 gap-y-1 p-3 bg-[#2a2e3a] rounded-[6px] mb-2 items-start",
                        match note.id.clone() {
                            Some(id) => rsx! {
                                p { class: "text-[#ccc] text-sm col-span-1", "Note ID: {id}" }
                            },
                            None => rsx! {
                                p { class: "text-[#ccc] text-sm col-span-1", "Note ID: (pending)" }
                            },
                        }
                        p { class: "text-[#ccc] text-sm col-span-1", "Snippets: {note.snippet_count}" }

                        match note.id.clone() {
                            Some(id) => rsx! {
                                button {
                                    class: "col-start-2 row-span-full self-center py-1.5 px-3 bg-[#dc3545] text-white border-none rounded cursor-pointer text-xs transition-colors duration-200 hover:bg-[#c82333]",
                                    onclick: move |_| {
                                        store.read().delete_note_optimistic(id.clone());
                                    },
                                    "Delete"
                                }
                            },
                            None => rsx! {},
                        }
                    }
                }
            },
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct KnotsTabProps {
    handle_knot_created: EventHandler<()>,
}

#[component]
fn KnotsTab(props: KnotsTabProps) -> Element {
    let mut knots = use_resource(backend::read_knots);
    let mut show_knot_editor = use_signal(|| false);

    rsx! {
        if show_knot_editor() {
            KnotEditor {
                handle_created: move |_knot| {
                    show_knot_editor.set(false);
                    knots.restart();
                    props.handle_knot_created.call(());
                },
                handle_cancel: move |_| {
                    show_knot_editor.set(false);
                },
            }
        } else {
            button {
                class: "py-2.5 px-4 bg-[#28a745] text-white border-none rounded cursor-pointer mb-4 transition-colors duration-200 hover:bg-[#218838]",
                onclick: move |_| show_knot_editor.set(true),
                "Create New Knot"
            }

            match knots() {
                Some(Ok(knots_data)) => rsx! {
                    if knots_data.is_empty() {
                        p { class: "text-[#888] text-center py-10 px-5 italic", "No knots found" }
                    } else {
                        for knot in knots_data {
                            KnotItem {
                                key: "knot-{knot.id_.clone().unwrap_or_default()}",
                                knot,
                                handle_deleted: move || knots.restart(),
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    p { class: "text-[#dc3545] text-center p-5", "Error loading knots: {error}" }
                },
                None => rsx! {
                    p { class: "text-[#6d85c6] text-center p-5", "Loading knots..." }
                },
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct KnotItemProps {
    knot: Knot,
    handle_deleted: Callback<()>,
}

#[component]
fn KnotItem(props: KnotItemProps) -> Element {
    let Some(id) = props.knot.id_
    else {
        return rsx! {
            p { "Invalid knot" }
        };
    };

    rsx! {
        div { class: "grid grid-cols-[1fr_auto] grid-rows-[auto_auto_auto] gap-x-3 gap-y-1 p-3 bg-[#2a2e3a] rounded-[6px] mb-2 items-start",
            p { class: "text-[#ccc] text-sm col-span-1", "{props.knot.label}" }
            p { class: "text-white font-medium col-span-1", "{props.knot.intent}" }
            p { class: "text-[#ccc] text-sm col-span-1", "Notes: {props.knot.notes.len()}, Knots: {props.knot.knots.len()}" }

            button {
                class: "col-start-2 row-span-full self-center py-1.5 px-3 bg-[#dc3545] text-white border-none rounded cursor-pointer text-xs transition-colors duration-200 hover:bg-[#c82333]",
                onclick: {
                    let id = id.clone();
                    move |_| {
                        let id = id.clone();
                        async move {
                            match backend::delete_knot(id.clone(), false).await {
                                Ok(_) => props.handle_deleted.call(()),
                                Err(error) => {
                                    tracing::error!("Failed to delete knot {id}: {error:?}");
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
