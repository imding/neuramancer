use {
    backend::{read_knots, read_notes},
    dioxus::{logger::tracing, prelude::*},
    knowledge_space_web::{
        EdgeKind, GraphEdgeInput, GraphNodeInput, MeshKind, NodeKind, SpaceTier, set_graph_edges,
        set_graph_nodes, set_space_tier, start_bevy,
    },
    schema::{Knot, Note},
    std::collections::HashSet,
    ui::{GraphEditor, NoteCreator, use_knot_store, use_notes_store},
};

const KNOWLEDGE_SPACE_CSS: Asset = asset!("/assets/knowledge_space.css");

#[component]
pub fn KnowledgeSpace() -> Element {
    let handle_config = move |_| {};
    let bevy_started = use_signal(|| false);
    let store = use_notes_store();
    let knot_store = use_knot_store();
    let mut tier = use_signal(|| SpaceTierUi::Notes);
    let mut selected_note_id = use_signal(|| None::<String>);
    let mut selected_knot_id = use_signal(|| None::<String>);
    let mut notes_resource = use_resource(read_notes);
    let mut knots_resource = use_resource(read_knots);
    let note_vms = use_memo(move || store.read().state.read().items.clone());
    let store_knots = use_memo(move || {
        knot_store
            .read()
            .state
            .read()
            .items
            .iter()
            .filter_map(|vm| vm.knot.clone())
            .collect::<Vec<Knot>>()
    });
    let notes = use_memo(move || {
        notes_resource()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .cloned()
            .unwrap_or_default()
    });
    let _knots = use_memo(move || {
        knots_resource()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .cloned()
            .unwrap_or_default()
    });

    use_effect(move || {
        store.read().refresh();
        knot_store.read().refresh();
    });

    use_future(move || {
        let mut bevy_started = bevy_started;

        async move {
            if !bevy_started() {
                bevy_started.set(true);
                start_bevy("#bevy-render");
            }
        }
    });

    use_effect(move || {
        if selected_note_id().is_none() &&
            let Some(id) = note_vms().iter().filter_map(|note| note.id.clone()).next()
        {
            selected_note_id.set(Some(id));
        }
    });

    use_effect(move || {
        if *tier.read() == SpaceTierUi::Snippets {
            notes_resource.restart();
        }
    });

    use_effect(move || {
        if *tier.read() != SpaceTierUi::Snippets {
            return;
        }

        let notes = notes();
        if notes.is_empty() {
            return;
        }

        let current_id = selected_note_id();
        let has_match = current_id
            .as_deref()
            .map(|id| notes.iter().any(|note| note.id_.as_deref() == Some(id)))
            .unwrap_or(false);

        if !has_match && let Some(id) = notes.iter().filter_map(|note| note.id_.clone()).next() {
            selected_note_id.set(Some(id));
        }
    });

    use_effect(move || {
        if selected_knot_id().is_none() &&
            let Some(id) = store_knots()
                .iter()
                .filter_map(|knot| knot.id_.clone())
                .next()
        {
            selected_knot_id.set(Some(id));
        }
    });

    let nodes = use_memo({
        let selected_note_id = selected_note_id;
        let selected_knot_id = selected_knot_id;
        move || match *tier.read() {
            SpaceTierUi::Snippets => build_snippet_nodes(selected_note_id().as_deref(), &notes()),
            SpaceTierUi::Notes => build_note_nodes(&note_vms()),
            SpaceTierUi::KnotIntermediate => {
                build_knot_intermediate_nodes(selected_knot_id().as_deref(), &store_knots())
            }
            SpaceTierUi::KnotRoot => build_knot_root_nodes(&store_knots()),
        }
    });

    let edges = use_memo({
        let selected_note_id = selected_note_id;
        let selected_knot_id = selected_knot_id;
        move || match *tier.read() {
            SpaceTierUi::Snippets => build_snippet_edges(selected_note_id().as_deref(), &notes()),
            SpaceTierUi::Notes => build_note_edges(&store_knots()),
            SpaceTierUi::KnotIntermediate => {
                build_knot_intermediate_edges(selected_knot_id().as_deref(), &store_knots())
            }
            SpaceTierUi::KnotRoot => Vec::new(),
        }
    });

    use_effect(move || {
        let current_tier = match *tier.read() {
            SpaceTierUi::Snippets => SpaceTier::Snippet {
                note_id: selected_note_id().unwrap_or_default(),
            },
            SpaceTierUi::Notes => SpaceTier::Note {
                knot_id: selected_knot_id().unwrap_or_default(),
            },
            SpaceTierUi::KnotIntermediate => SpaceTier::KnotIntermediate {
                parent_knot_id: selected_knot_id().unwrap_or_default(),
            },
            SpaceTierUi::KnotRoot => SpaceTier::KnotRoot,
        };

        set_space_tier(current_tier);
        set_graph_nodes(nodes());
        set_graph_edges(edges());
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KNOWLEDGE_SPACE_CSS }

        div { id: "knowledge-space",
            canvas { id: "bevy-render" }

            div { id: "controls",

                div { id: "tier-controls",
                    button {
                        class: if *tier.read() == SpaceTierUi::Snippets { "active" } else { "" },
                        onclick: move |_| tier.set(SpaceTierUi::Snippets),
                        "Snippet Space"
                    }
                    button {
                        class: if *tier.read() == SpaceTierUi::Notes { "active" } else { "" },
                        onclick: move |_| tier.set(SpaceTierUi::Notes),
                        "Note Space"
                    }
                    button {
                        class: if *tier.read() == SpaceTierUi::KnotIntermediate { "active" } else { "" },
                        onclick: move |_| tier.set(SpaceTierUi::KnotIntermediate),
                        "Knot Intermediate"
                    }
                    button {
                        class: if *tier.read() == SpaceTierUi::KnotRoot { "active" } else { "" },
                        onclick: move |_| tier.set(SpaceTierUi::KnotRoot),
                        "Knot Root"
                    }
                }

                br {}

                button { onclick: handle_config,

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
                        notes_resource.restart();
                        knots_resource.restart();
                        store.read().refresh();
                        knot_store.read().refresh();
                    }
                }

                GraphEditor {
                    handle_updated: move |_| {
                        notes_resource.restart();
                        knots_resource.restart();
                        store.read().refresh();
                        knot_store.read().refresh();
                    }
                }

                br {}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SpaceTierUi {
    Snippets,
    Notes,
    KnotIntermediate,
    KnotRoot,
}

fn build_snippet_nodes(_selected_note_id: Option<&str>, notes: &[Note]) -> Vec<GraphNodeInput> {
    notes
        .iter()
        .enumerate()
        .flat_map(|(note_index, note)| {
            let note_id = note
                .id_
                .clone()
                .unwrap_or(format!("note:unknown:{note_index}"));

            note.snippets
                .iter()
                .enumerate()
                .map(move |(index, snippet)| {
                    let id = snippet
                        .id_
                        .clone()
                        .unwrap_or(format!("snippet:{note_id}:{index}"));

                    (note_id.clone(), id)
                })
        })
        .enumerate()
        .map(|(index, (note_id, snippet_id))| GraphNodeInput {
            id: snippet_id,
            kind: NodeKind::Snippet,
            group_id: Some(note_id),
            position: grid_position(index, 10, 1.4, 0.6),
            mesh_kind: MeshKind::Sphere,
            color: [118, 167, 255, 255],
        })
        .collect()
}

fn build_snippet_edges(_selected_note_id: Option<&str>, notes: &[Note]) -> Vec<GraphEdgeInput> {
    notes
        .iter()
        .enumerate()
        .map(|(note_index, note)| {
            let note_id = note
                .id_
                .clone()
                .unwrap_or(format!("note:unknown:{note_index}"));

            note.snippets
                .iter()
                .enumerate()
                .map(|(index, snippet)| {
                    snippet
                        .id_
                        .clone()
                        .unwrap_or(format!("snippet:{note_id}:{index}"))
                })
                .collect::<Vec<String>>()
        })
        .flat_map(|ids| pairwise_edges(&ids, EdgeKind::NoteMembership, false))
        .collect()
}

fn build_note_nodes(note_vms: &[ui::NoteVm]) -> Vec<GraphNodeInput> {
    note_vms
        .iter()
        .filter_map(|note| note.id.clone())
        .enumerate()
        .map(|(index, id)| {
            let position = grid_position(index, 8, 2.2, 0.8);
            GraphNodeInput {
                id,
                kind: NodeKind::Note,
                group_id: None,
                position,
                mesh_kind: MeshKind::Cube,
                color: [120, 210, 165, 255],
            }
        })
        .collect()
}

fn build_note_edges(knots: &[Knot]) -> Vec<GraphEdgeInput> {
    let mut edges = Vec::new();
    for knot in knots {
        let note_ids: Vec<String> = knot
            .notes
            .iter()
            .filter_map(|note| note.id_.clone())
            .collect();
        edges.extend(pairwise_edges(&note_ids, EdgeKind::KnotMembership, true));
    }
    edges
}

fn build_knot_intermediate_nodes(
    selected_knot_id: Option<&str>,
    knots: &[Knot],
) -> Vec<GraphNodeInput> {
    let Some(knot_id) = selected_knot_id
    else {
        return Vec::new();
    };

    let Some(parent) = knots
        .iter()
        .find(|knot| knot.id_.as_deref() == Some(knot_id))
    else {
        return Vec::new();
    };

    let mut nodes = Vec::new();

    nodes.push(GraphNodeInput {
        id: knot_id.to_string(),
        kind: NodeKind::Knot,
        group_id: None,
        position: [0.0, 0.9, 0.0],
        mesh_kind: MeshKind::Capsule,
        color: [252, 200, 114, 255],
    });

    let total = parent.knots.len().max(1);
    for (index, child) in parent.knots.iter().enumerate() {
        let Some(id) = child.id_.clone()
        else {
            continue;
        };
        let position = ring_position(index, total, 4.2, 0.8);
        nodes.push(GraphNodeInput {
            id,
            kind: NodeKind::Knot,
            group_id: Some(knot_id.to_string()),
            position,
            mesh_kind: MeshKind::Capsule,
            color: [255, 170, 96, 255],
        });
    }

    nodes
}

fn build_knot_intermediate_edges(
    selected_knot_id: Option<&str>,
    knots: &[Knot],
) -> Vec<GraphEdgeInput> {
    let Some(knot_id) = selected_knot_id
    else {
        return Vec::new();
    };
    let Some(parent) = knots
        .iter()
        .find(|knot| knot.id_.as_deref() == Some(knot_id))
    else {
        return Vec::new();
    };

    parent
        .knots
        .iter()
        .filter_map(|child| child.id_.clone())
        .map(|child_id| GraphEdgeInput {
            from: knot_id.to_string(),
            to: child_id,
            kind: EdgeKind::ParentChild,
            visible: true,
        })
        .collect()
}

fn build_knot_root_nodes(knots: &[Knot]) -> Vec<GraphNodeInput> {
    let mut child_ids = HashSet::new();
    for knot in knots {
        for child in knot.knots.iter() {
            if let Some(id) = child.id_.clone() {
                child_ids.insert(id);
            }
        }
    }

    knots
        .iter()
        .filter_map(|knot| knot.id_.clone())
        .filter(|id| !child_ids.contains(id))
        .enumerate()
        .map(|(index, id)| GraphNodeInput {
            id,
            kind: NodeKind::Knot,
            group_id: None,
            position: grid_position(index, 5, 3.0, 1.0),
            mesh_kind: MeshKind::Capsule,
            color: [250, 186, 120, 255],
        })
        .collect()
}

fn pairwise_edges(ids: &[String], kind: EdgeKind, visible: bool) -> Vec<GraphEdgeInput> {
    let mut edges = Vec::new();
    for (i, from) in ids.iter().enumerate() {
        for to in ids.iter().skip(i + 1) {
            edges.push(GraphEdgeInput {
                from: from.clone(),
                to: to.clone(),
                kind: kind.clone(),
                visible,
            });
        }
    }
    edges
}

fn grid_position(index: usize, per_row: usize, spacing: f32, height: f32) -> [f32; 3] {
    let row = index / per_row;
    let col = index % per_row;
    let x = col as f32 * spacing - (per_row as f32 - 1.0) * spacing * 0.5;
    let z = row as f32 * spacing;
    [x, height, z]
}

fn ring_position(index: usize, total: usize, radius: f32, height: f32) -> [f32; 3] {
    let angle = (index as f32 / total as f32) * std::f32::consts::TAU;
    let x = radius * angle.cos();
    let z = radius * angle.sin();
    [x, height, z]
}
