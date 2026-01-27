use knowledge_space_core::KnowledgeSpacePlugin;

#[cfg(target_arch = "wasm32")]
use bevy::{
    prelude::*,
    window::{Window, WindowPlugin},
};

#[cfg(target_arch = "wasm32")]
pub fn start_bevy(canvas_selector: &str) {
    App::new()
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

#[cfg(not(target_arch = "wasm32"))]
pub fn start_bevy(_canvas_selector: &str) {
    // No-op on non-wasm targets.
}
