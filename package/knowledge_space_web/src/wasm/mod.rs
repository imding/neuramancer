mod plugin;

use {
    bevy::{
        prelude::*,
        window::{Window, WindowPlugin},
    },
    knowledge_space_core::{GraphEdgeInput, GraphInputSnapshot, GraphNodeInput, SpaceTier},
    once_cell::sync::Lazy,
    plugin::KnowledgeSpacePlugin,
    std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    web_sys::{Event, window},
};

/// Shared state between the Dioxus UI and the Bevy scene.
///
/// Wraps the core [`GraphInputSnapshot`] in an `Arc<Mutex<...>>` and derives
/// [`Resource`] so it can be inserted into the Bevy world.
#[derive(Resource, Clone)]
pub struct GraphInputState {
    pub shared: Arc<Mutex<GraphInputSnapshot>>,
}

impl Default for GraphInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphInputState {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(GraphInputSnapshot::default())),
        }
    }
}

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
        .add_systems(Startup, mark_bevy_ready)
        .run();
}

fn mark_bevy_ready() {
    let Some(window) = window()
    else {
        return;
    };
    let Ok(event) = Event::new("bevy:ready")
    else {
        return;
    };

    let _ = window.dispatch_event(&event);
}

fn update_snapshot(update: impl FnOnce(&mut GraphInputSnapshot)) {
    match SHARED_STATE.shared.lock() {
        Ok(mut snapshot) => update(&mut snapshot),
        Err(poisoned) => update(&mut poisoned.into_inner()),
    }
}
