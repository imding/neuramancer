#[cfg(all(target_arch = "wasm32", feature = "bevy"))]
mod wasm;

#[cfg(not(all(target_arch = "wasm32", feature = "bevy")))]
mod native;

#[cfg(all(target_arch = "wasm32", feature = "bevy"))]
pub use wasm::{set_graph_edges, set_graph_nodes, set_space_tier, start_bevy};

#[cfg(not(all(target_arch = "wasm32", feature = "bevy")))]
pub use native::{set_graph_edges, set_graph_nodes, set_space_tier, start_bevy};

pub use knowledge_space_core::{
    EdgeKind, GraphEdgeInput, GraphNodeInput, MeshKind, NodeKind, SpaceTier, builder,
};
