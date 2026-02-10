pub mod builder;

use {
    bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*},
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};

pub struct KnowledgeSpacePlugin;

impl Plugin for KnowledgeSpacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphAppliedSnapshot::default())
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (sync_graph_nodes, draw_graph_edges));
    }
}

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

#[derive(Resource, Clone, Debug, PartialEq)]
struct GraphAppliedSnapshot {
    tier: SpaceTier,
    nodes: Vec<GraphNodeInput>,
    edges: Vec<GraphEdgeInput>,
}

impl Default for GraphAppliedSnapshot {
    fn default() -> Self {
        Self {
            tier: SpaceTier::KnotRoot,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Resource, Clone)]
struct GraphMeshAssets {
    sphere: Handle<Mesh>,
    cube: Handle<Mesh>,
    capsule: Handle<Mesh>,
}

#[derive(Component)]
struct SpaceEntity;

#[derive(Component)]
struct GraphNodeId(String);

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_assets = GraphMeshAssets {
        sphere: meshes.add(Sphere::new(0.45).mesh().ico(5).unwrap()),
        cube: meshes.add(Cuboid::new(0.9, 0.9, 0.9)),
        capsule: meshes.add(Capsule3d::new(0.3, 0.5)),
    };

    commands.insert_resource(mesh_assets);

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(-3.0, 5.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(8.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(245, 245, 245))),
        Transform::from_rotation(Quat::from_rotation_x(-90f32.to_radians())),
    ));
}

fn sync_graph_nodes(
    mut commands: Commands,
    shared_state: Option<Res<GraphInputState>>,
    mut applied: ResMut<GraphAppliedSnapshot>,
    mesh_assets: Res<GraphMeshAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, With<SpaceEntity>>,
) {
    let Some(shared_state) = shared_state
    else {
        return;
    };

    let snapshot = match shared_state.shared.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };

    if snapshot.tier == applied.tier &&
        snapshot.nodes == applied.nodes &&
        snapshot.edges == applied.edges
    {
        return;
    }

    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    for node in snapshot.nodes.iter() {
        let mesh = match node.mesh_kind {
            MeshKind::Sphere => mesh_assets.sphere.clone(),
            MeshKind::Cube => mesh_assets.cube.clone(),
            MeshKind::Capsule => mesh_assets.capsule.clone(),
        };
        let color = Color::srgba_u8(node.color[0], node.color[1], node.color[2], node.color[3]);
        let position = Vec3::new(node.position[0], node.position[1], node.position[2]);
        let graph_node_id = GraphNodeId(node.id.clone());

        println!("GraphNodeId: {}", graph_node_id.0);

        commands.spawn((
            SpaceEntity,
            graph_node_id,
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(color)),
            Transform::from_translation(position),
        ));
    }

    applied.tier = snapshot.tier;
    applied.nodes = snapshot.nodes;
    applied.edges = snapshot.edges;
}

fn draw_graph_edges(mut gizmos: Gizmos, applied: Res<GraphAppliedSnapshot>) {
    if matches!(applied.tier, SpaceTier::KnotRoot) {
        return;
    }

    let mut positions: HashMap<&str, Vec3> = HashMap::new();

    for node in applied.nodes.iter() {
        positions.insert(
            node.id.as_str(),
            Vec3::new(node.position[0], node.position[1], node.position[2]),
        );
    }

    for edge in applied.edges.iter() {
        if !edge.visible {
            continue;
        }

        let Some(from) = positions.get(edge.from.as_str())
        else {
            continue;
        };
        let Some(to) = positions.get(edge.to.as_str())
        else {
            continue;
        };

        let color = match edge.kind {
            EdgeKind::NoteMembership => Color::srgb_u8(120, 168, 255),
            EdgeKind::KnotMembership => Color::srgb_u8(120, 210, 165),
            EdgeKind::ParentChild => Color::srgb_u8(252, 200, 114),
        };

        gizmos.line(*from, *to, color);
    }
}
