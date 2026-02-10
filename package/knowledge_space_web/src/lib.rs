#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
pub use wasm::{set_graph_edges, set_graph_nodes, set_space_tier, start_bevy};

#[cfg(not(target_arch = "wasm32"))]
pub use native::{set_graph_edges, set_graph_nodes, set_space_tier, start_bevy};

pub use knowledge_space_core::{
    EdgeKind, GraphEdgeInput, GraphNodeInput, MeshKind, NodeKind, SpaceTier, builder,
};
