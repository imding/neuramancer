use {
    dioxus::{logger::tracing, prelude::*},
    knowledge_space_web::{
        SpaceTier,
        builder::{
            KnotInput, NoteInput, SnippetInput, build_knot_intermediate_edges,
            build_knot_intermediate_nodes, build_knot_root_nodes, build_note_edges,
            build_note_nodes, build_snippet_edges, build_snippet_nodes,
        },
        set_graph_edges, set_graph_nodes, set_space_tier,
    },
    ui::{GraphEditor, KnotStore, NoteCreator, use_knot_store, use_notes_store},
};

#[derive(Clone, Copy, PartialEq)]
enum SpaceTierUi {
    Snippets,
    Notes,
    KnotIntermediate,
    KnotRoot,
}

fn tier_class(active: bool) -> &'static str {
    if active {
        "bg-[#1f1f1f] border border-[#1f1f1f] text-white rounded-full py-[0.45rem] px-[0.9rem] font-semibold cursor-pointer"
    } else {
        "bg-white/95 border border-black/20 rounded-full py-[0.45rem] px-[0.9rem] font-semibold cursor-pointer"
    }
}

#[component]
pub fn KnowledgeSpace() -> Element {
    let handle_config = move |_| {};
    let store = use_notes_store();
    let knot_store = use_knot_store();
    let mut tier = use_signal(|| SpaceTierUi::Notes);
    let graph = use_memo(move || match *tier.read() {
        SpaceTierUi::Snippets => {
            let inputs: Vec<SnippetInput> = store
                .read()
                .state
                .read()
                .items
                .iter()
                .map(|vm| SnippetInput {
                    note_id: vm.id.clone().unwrap_or_else(|| vm.local_key.clone()),
                    snippet_count: vm.snippet_count,
                })
                .collect();

            (build_snippet_nodes(&inputs), build_snippet_edges(&inputs))
        }
        SpaceTierUi::Notes => {
            let note_inputs: Vec<NoteInput> = store
                .read()
                .state
                .read()
                .items
                .iter()
                .map(|vm| NoteInput {
                    id: vm.id.clone().unwrap_or(vm.local_key.clone()),
                    snippet_count: vm.snippet_count,
                })
                .collect();
            let knot_inputs = map_knot_inputs(&knot_store);

            (
                build_note_nodes(&note_inputs),
                build_note_edges(&knot_inputs),
            )
        }
        SpaceTierUi::KnotIntermediate => {
            let ki = map_knot_inputs(&knot_store);

            (
                build_knot_intermediate_nodes(&ki),
                build_knot_intermediate_edges(&ki),
            )
        }
        SpaceTierUi::KnotRoot => {
            let ki = map_knot_inputs(&knot_store);
            (build_knot_root_nodes(&ki), Vec::new())
        }
    });

    use_effect(move || {
        let current_tier = match *tier.read() {
            SpaceTierUi::Snippets => SpaceTier::Snippet,
            SpaceTierUi::Notes => SpaceTier::Note,
            SpaceTierUi::KnotIntermediate => SpaceTier::KnotIntermediate,
            SpaceTierUi::KnotRoot => SpaceTier::KnotRoot,
        };
        let (nodes, edges) = graph();

        set_space_tier(current_tier);
        set_graph_nodes(nodes);
        set_graph_edges(edges);
    });

    rsx! {
        div { class: "absolute top-0 w-screen h-screen",
            div { class: "grid grid-cols-[1fr_80px_120px_80px_1fr] items-end gap-8 absolute bottom-0 w-screen pb-4",
                div { class: "absolute left-0 right-0 bottom-[calc(100%+0.75rem)] flex flex-wrap justify-center gap-2",
                    button {
                        class: tier_class(*tier.read() == SpaceTierUi::Snippets),
                        onclick: move |_| tier.set(SpaceTierUi::Snippets),
                        "Snippet Space"
                    }
                    button {
                        class: tier_class(*tier.read() == SpaceTierUi::Notes),
                        onclick: move |_| tier.set(SpaceTierUi::Notes),
                        "Note Space"
                    }
                    button {
                        class: tier_class(*tier.read() == SpaceTierUi::KnotIntermediate),
                        onclick: move |_| tier.set(SpaceTierUi::KnotIntermediate),
                        "Knot Intermediate"
                    }
                    button {
                        class: tier_class(*tier.read() == SpaceTierUi::KnotRoot),
                        onclick: move |_| tier.set(SpaceTierUi::KnotRoot),
                        "Knot Root"
                    }
                }

                br {}

                button {
                    class: "bg-white border-none p-4 cursor-pointer rounded-full [&_svg]:w-full [&_svg]:h-full",
                    onclick: handle_config,
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        width: "512",
                        height: "512",
                        view_box: "0 0 512 512",

                        path {
                            fill: "currentColor",
                            d: "m235.045 25.752l-41.775 8.215l6.894 35.947c-19.303 5.91-37.997 14.43-54.643 25.852l-23.834-27.78L89.04 96.158l24.433 27.988c-13.495 14.454-25.328 31.203-34.16 49.78L44.64 161.813l-14.203 40.652l34.674 12.11c-4.54 19.832-5.6 40.113-4.057 59.624l-36.547 6.685l8.006 42.37l35.947-6.896c5.91 19.303 15.235 37.61 26.657 54.255L67.34 394.45l27.572 32.44l27.78-23.837c14.537 13.64 31.05 25.255 49.78 34.158l-12.115 34.675l40.653 14.2l12.11-34.67c20.202 4.695 40.354 5.885 60.222 4.267l6.894 35.947l41.774-8.214l-6.895-35.947c19.304-5.912 38-14.43 54.645-25.853l23.836 27.778l32.646-28.17l-24.433-27.99c13.355-14.305 25.153-30.836 33.948-49.18l34.674 12.113l14.2-40.652l-34.673-12.112c4.654-20.034 5.825-40.508 4.27-60.22l36.543-6.688l-8.003-42.37l-35.948 6.894c-5.91-19.304-15.237-37.608-26.66-54.254l27.78-23.836l-27.573-32.438l-27.78 23.836c-14.538-13.64-31.05-25.257-49.78-34.16l11.905-34.076l-40.65-14.2v.003L302.16 65.97c-20.2-4.698-40.35-5.887-60.22-4.27zm23.178 87.603c28.01.105 56.29 8.287 81.183 25.24c59.008 40.186 79.122 116.127 51.112 179.13l-57.065-38.862c8.25-31.382-3.378-65.89-31.715-85.19c-28.34-19.3-64.227-17.152-90.408 2.02l-57.316-39.033c27.96-28.41 65.826-43.448 104.21-43.305zM124.7 199.582l56.89 38.744c-8.423 31.474 2.948 66.307 31.39 85.676c28.44 19.37 65.5 17.52 91.7-1.848l56.625 38.565c-48.347 49.327-126.52 58.51-185.614 18.266C116.6 338.742 96.51 262.64 124.7 199.582m132.87 14.29a44.75 44.75 0 0 1 25.102 7.798c20.516 13.973 25.83 41.964 11.857 62.48c-13.972 20.517-41.967 25.827-62.483 11.856s-25.827-41.964-11.856-62.48c8.734-12.823 22.942-19.707 37.38-19.655z",
                        }
                    }
                }

                NoteCreator {
                    handle_created: move |note| {
                        tracing::debug!("{note:?}");
                    }
                }

                GraphEditor {
                    handle_updated: move |_| {
                        knot_store.read().refresh();
                    }
                }

                br {}
            }
        }
    }
}

fn map_knot_inputs(knot_store: &Signal<KnotStore>) -> Vec<KnotInput> {
    knot_store
        .read()
        .state
        .read()
        .items
        .iter()
        .map(|vm| KnotInput {
            id: vm.id.clone().unwrap_or_else(|| vm.local_key.clone()),
            label: vm.knot.as_ref().map(|knot| knot.label.clone()),
            child_knot_ids: vm
                .knot
                .as_ref()
                .map(|knot| {
                    knot.knots
                        .iter()
                        .filter_map(|child| child.id_.clone())
                        .collect()
                })
                .unwrap_or_default(),
            note_ids: vm
                .knot
                .as_ref()
                .map(|knot| {
                    knot.notes
                        .iter()
                        .filter_map(|note| note.id_.clone())
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}
