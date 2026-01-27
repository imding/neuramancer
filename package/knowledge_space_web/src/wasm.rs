use {
    bevy::{
        prelude::*,
        window::{Window, WindowPlugin},
    },
    knowledge_space_core::{KnowledgeSpacePlugin, KnowledgeSpaceState},
    once_cell::sync::Lazy,
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

static BEVY_STARTED: AtomicBool = AtomicBool::new(false);

static SHARED_STATE: Lazy<KnowledgeSpaceState> = Lazy::new(|| KnowledgeSpaceState {
    note_count: Arc::new(Mutex::new(0)),
});

pub fn set_note_count(count: usize) {
    match SHARED_STATE.note_count.lock() {
        Ok(mut value) => *value = count,
        Err(poisoned) => *poisoned.into_inner() = count,
    }
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
