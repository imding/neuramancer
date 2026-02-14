pub mod builder;

#[derive(Clone, Debug, PartialEq)]
pub enum SpaceTier {
    Snippet,
    Note,
    KnotIntermediate,
    KnotRoot,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Snippet,
    Note,
    Knot,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MeshKind {
    Sphere,
    Cube,
    Capsule,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EdgeKind {
    NoteMembership,
    KnotMembership,
    ParentChild,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeInput {
    pub id: String,
    pub kind: NodeKind,
    pub group_id: Option<String>,
    pub position: [f32; 3],
    pub mesh_kind: MeshKind,
    pub color: [u8; 4],
    pub glyph: Option<glyph_core::Glyph>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdgeInput {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphInputSnapshot {
    pub tier: SpaceTier,
    pub nodes: Vec<GraphNodeInput>,
    pub edges: Vec<GraphEdgeInput>,
}

impl Default for GraphInputSnapshot {
    fn default() -> Self {
        Self {
            tier: SpaceTier::KnotRoot,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}
