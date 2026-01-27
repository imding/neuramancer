use {
    bevy::{
        prelude::*,
        window::{Window, WindowPlugin},
    },
    knowledge_space_core::{
        GraphEdgeInput, GraphInputSnapshot, GraphInputState, GraphNodeInput, KnowledgeSpacePlugin,
        SpaceTier,
    },
    once_cell::sync::Lazy,
    std::sync::atomic::{AtomicBool, Ordering},
};

static BEVY_STARTED: AtomicBool = AtomicBool::new(false);

static SHARED_STATE: Lazy<GraphInputState> = Lazy::new(GraphInputState::new);

pub fn set_space_tier(tier: SpaceTier) {
    update_snapshot(|snapshot| {
        snapshot.tier = tier;
    });
}

pub fn set_graph_nodes(nodes: Vec<GraphNodeInput>) {
    update_snapshot(|snapshot| {
        snapshot.nodes = nodes;
    });
}

pub fn set_graph_edges(edges: Vec<GraphEdgeInput>) {
    update_snapshot(|snapshot| {
        snapshot.edges = edges;
    });
}

pub fn start_bevy(canvas_selector: &str) {
    if BEVY_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    App::new()
        .insert_resource(SHARED_STATE.clone())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some(canvas_selector.to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(KnowledgeSpacePlugin)
        .run();
}

fn update_snapshot(update: impl FnOnce(&mut GraphInputSnapshot)) {
    match SHARED_STATE.shared.lock() {
        Ok(mut snapshot) => update(&mut snapshot),
        Err(poisoned) => update(&mut poisoned.into_inner()),
    }
}
