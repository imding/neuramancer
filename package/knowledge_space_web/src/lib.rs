#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
pub use wasm::{set_note_count, start_bevy};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{set_note_count, start_bevy};
