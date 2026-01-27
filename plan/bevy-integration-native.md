1. Code Structure Analysis

File Overview

The example consists of 5 key files that work together:

| File | Purpose | Key Responsibilities |
|------|---------|---------------------|
| `Cargo.toml` | Dependencies & features | Defines workspace dependencies for Bevy, Dioxus, and WGPU |
| `main.rs` | Entry point & UI | Dioxus app, UI components, state management, canvas integration |
| `demo_renderer.rs` | Communication bridge | Message passing between Dioxus and Bevy via channels |
| `bevy_renderer.rs` | Bevy initialization | Creates headless Bevy app, manages texture rendering |
| `bevy_scene_plugin.rs` | 3D scene logic | Bevy systems for spawning entities, animation, responding to UI |
| `styles.css` | Layout & styling | CSS for canvas positioning and UI overlay |

Architecture Flow

┌─────────────────────────────────────────────────────────────┐
│                    Dioxus Application                        │
│  ┌────────────────────────────────────────────────────┐     │
│  │  main.rs: UI Components                            │     │
│  │  - SpinningCube component                          │     │
│  │  - use_wgpu() hook                                 │     │
│  │  - Signals for state (color, show_cube)           │     │
│  └──────────────┬─────────────────────────────────────┘     │
│                 │ Creates & registers                        │
│                 ▼                                            │
│  ┌────────────────────────────────────────────────────┐     │
│  │  demo_renderer.rs: DemoPaintSource                 │     │
│  │  - Implements CustomPaintSource trait              │     │
│  │  - MPSC channel for messages                       │     │
│  │  - Manages renderer lifecycle                      │     │
│  └──────────────┬─────────────────────────────────────┘     │
│                 │ Creates on resume()                        │
│                 ▼                                            │
│  ┌────────────────────────────────────────────────────┐     │
│  │  bevy_renderer.rs: BevyRenderer                    │     │
│  │  - Headless Bevy App                               │     │
│  │  - Texture management                              │     │
│  │  - Shares WGPU device with Dioxus                  │     │
│  └──────────────┬─────────────────────────────────────┘     │
│                 │ Adds plugin                                │
│                 ▼                                            │
│  ┌────────────────────────────────────────────────────┐     │
│  │  bevy_scene_plugin.rs: BevyScenePlugin             │     │
│  │  - Bevy systems (setup, animate, update_color)    │     │
│  │  - 3D entities (cube, camera, lights)             │     │
│  │  - Reads UIData resource                          │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘


─────────────────────────────────────────────────────────────────

2. Step-by-Step Implementation for Neuramancer

Phase 1: Project Structure Setup

Directory Structure

neuramancer/
├── Cargo.toml                    # Workspace root
├── ui/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── knowledge_graph/      # NEW: 3D visualization module
│       │   ├── mod.rs
│       │   ├── bevy_renderer.rs
│       │   ├── graph_scene_plugin.rs
│       │   └── graph_paint_source.rs
│       └── components/
│           └── knowledge_graph_view.rs  # NEW: Dioxus component
├── schema/
│   └── src/
│       └── graph.rs              # NEW: Graph-specific types
└── web/
   └── src/
       └── views/
           └── graph.rs          # NEW: Graph route

Step 1.1: Update `neuramancer/Cargo.toml`

Add Bevy to workspace dependencies:

[workspace.dependencies]
# ... existing dependencies ...
bevy = { version = "0.17", default-features = false, features = [
   "bevy_render",
   "bevy_core_pipeline",
   "bevy_pbr",
   "bevy_asset",
] }
wgpu = "25"

Step 1.2: Update `neuramancer/ui/Cargo.toml`

[dependencies]
# ... existing dependencies ...
bevy = { workspace = true }
wgpu = { workspace = true }
[features]
default = []
desktop = ["dioxus/desktop", "dioxus-native"]
native = ["dioxus/native", "dioxus-native"]
[dependencies.dioxus-native]
workspace = true
optional = true


───────────────────────────────

Phase 2: Core Integration Files

File 1: `ui/src/knowledge_graph/mod.rs`

//! Knowledge graph 3D visualization module
//! 
//! This module integrates Bevy 3D rendering with Dioxus UI for visualizing
//! the knowledge graph as an interactive 3D scene.
mod bevy_renderer;
mod graph_scene_plugin;
mod graph_paint_source;
pub use bevy_renderer::BevyGraphRenderer;
pub use graph_scene_plugin::{GraphScenePlugin, GraphData};
pub use graph_paint_source::{GraphPaintSource, GraphMessage};
// Re-export for convenience
pub use bevy::prelude::*;

File 2: `ui/src/knowledge_graph/graph_paint_source.rs`

This is the communication bridge between Dioxus and Bevy:

use crate::knowledge_graph::bevy_renderer::BevyGraphRenderer;
use dioxus_native::{CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle};
use schema::{Note, Snippet};
use std::sync::mpsc::{channel, Receiver, Sender};
/// Messages sent from Dioxus UI to Bevy renderer
#[derive(Debug, Clone)]
pub enum GraphMessage {
   /// Add a new node to the graph
   AddNode { id: String, position: [f32; 3], label: String },
   /// Add an edge between two nodes
   AddEdge { from: String, to: String },
   /// Remove a node
   RemoveNode { id: String },
   /// Update node position
   UpdateNodePosition { id: String, position: [f32; 3] },
   /// Highlight a node (e.g., on hover)
   HighlightNode { id: Option<String> },
   /// Update camera position
   SetCameraPosition { position: [f32; 3], look_at: [f32; 3] },
}
enum RendererState {
   Active(Box<BevyGraphRenderer>),
   Suspended,
}
pub struct GraphPaintSource {
   state: RendererState,
   start_time: std::time::Instant,
   tx: Sender<GraphMessage>,
   rx: Receiver<GraphMessage>,
   pending_messages: Vec<GraphMessage>,
}
impl GraphPaintSource {
   pub fn new() -> Self {
       let (tx, rx) = channel();
       Self {
           state: RendererState::Suspended,
           start_time: std::time::Instant::now(),
           tx,
           rx,
           pending_messages: Vec::new(),
       }
   }
   /// Get a sender for sending messages to the renderer
   pub fn sender(&self) -> Sender<GraphMessage> {
       self.tx.clone()
   }
   /// Process all pending messages from the channel
   fn process_messages(&mut self) {
       loop {
           match self.rx.try_recv() {
               Err(_) => break,
               Ok(msg) => {
                   self.pending_messages.push(msg);
               }
           }
       }
   }
   fn render(
       &mut self,
       ctx: CustomPaintCtx<'_>,
       width: u32,
       height: u32,
   ) -> Option<TextureHandle> {
       if width == 0 || height == 0 {
           return None;
       }

       let RendererState::Active(renderer) = &mut self.state else {
           return None;
       };
       // Pass pending messages to renderer
       let messages = std::mem::take(&mut self.pending_messages);
       renderer.render(ctx, messages, width, height, &self.start_time)
   }
}
impl CustomPaintSource for GraphPaintSource {
   fn resume(&mut self, device_handle: &DeviceHandle) {
       let renderer = BevyGraphRenderer::new(device_handle);
       self.state = RendererState::Active(Box::new(renderer));
   }
   fn suspend(&mut self) {
       self.state = RendererState::Suspended;
   }
   fn render(
       &mut self,
       ctx: CustomPaintCtx<'_>,
       width: u32,
       height: u32,
       _scale: f64,
   ) -> Option<TextureHandle> {
       self.process_messages();
       self.render(ctx, width, height)
   }
}

File 3: `ui/src/knowledge_graph/bevy_renderer.rs`

This creates and manages the headless Bevy app:

use crate::knowledge_graph::{GraphScenePlugin, GraphData, GraphMessage};
use bevy::{
   camera::{ManualTextureViewHandle, RenderTarget},
   prelude::*,
   render::{
       render_resource::TextureFormat,
       renderer::{
           RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue,
           WgpuWrapper,
       },
       settings::{RenderCreation, RenderResources},
       texture::ManualTextureView,
       RenderPlugin,
   },
};
use dioxus_native::{CustomPaintCtx, DeviceHandle, TextureHandle};
use std::sync::Arc;
pub struct BevyGraphRenderer {
   app: App,
   wgpu_device: wgpu::Device,
   last_texture_size: (u32, u32),
   texture_handle: Option<TextureHandle>,
   manual_texture_view_handle: Option<ManualTextureViewHandle>,
}
impl BevyGraphRenderer {
   pub fn new(device_handle: &DeviceHandle) -> Self {
       // Create a headless Bevy App that shares WGPU resources with Dioxus
       let mut app = App::new();

       app.add_plugins(
           DefaultPlugins
               .set(RenderPlugin {
                   // CRITICAL: Reuse Dioxus's WGPU device instead of creating new one
                   render_creation: RenderCreation::Manual(RenderResources(
                       RenderDevice::new(WgpuWrapper::new(device_handle.device.clone())),
                       RenderQueue(Arc::new(WgpuWrapper::new(device_handle.queue.clone()))),
                       RenderAdapterInfo(WgpuWrapper::new(device_handle.adapter.get_info())),
                       RenderAdapter(Arc::new(WgpuWrapper::new(device_handle.adapter.clone()))),
                       RenderInstance(Arc::new(WgpuWrapper::new(device_handle.instance.clone()))),
                   )),
                   synchronous_pipeline_compilation: true,
                   ..default()
               })
               .set(WindowPlugin {
                   primary_window: None,  // Headless - no window
                   exit_condition: bevy::window::ExitCondition::DontExit,
                   close_when_requested: false,
                   ..Default::default()
               })
               .disable::<bevy::winit::WinitPlugin>(),  // No winit for headless
       );
       // Setup rendering to texture
       app.insert_resource(ManualTextureViews::default());
       // Add graph data resource (will be updated from Dioxus)
       app.insert_resource(GraphData::default());
       // Add the knowledge graph scene plugin
       app.add_plugins(GraphScenePlugin);
       // Initialize the app
       app.finish();
       app.cleanup();
       Self {
           app,
           wgpu_device: device_handle.device.clone(),
           last_texture_size: (0, 0),
           texture_handle: None,
           manual_texture_view_handle: None,
       }
   }
   pub fn render(
       &mut self,
       ctx: CustomPaintCtx<'_>,
       messages: Vec<GraphMessage>,
       width: u32,
       height: u32,
       _start_time: &std::time::Instant,
   ) -> Option<TextureHandle> {
       // Update graph data from messages
       if let Some(mut graph_data) = self.app.world_mut().get_resource_mut::<GraphData>() {
           for msg in messages {
               graph_data.process_message(msg);
           }
       }
       // Initialize or resize texture if needed
       self.init_texture(ctx, width, height);

       // Run one frame of Bevy to render the scene
       self.app.update();
       self.texture_handle.clone()
   }
   fn init_texture(&mut self, mut ctx: CustomPaintCtx<'_>, width: u32, height: u32) {
       // Reuse texture if size hasn't changed
       let current_size = (width, height);
       if self.texture_handle.is_some() && self.last_texture_size == current_size {
           return;
       }
       let world = self.app.world_mut();
       // Skip if no camera exists yet
       if world.query::<&Camera>().single(world).is_err() {
           return;
       }
       if let Some(mut manual_texture_views) = world.get_resource_mut::<ManualTextureViews>() {
           // Clean up previous texture
           if self.texture_handle.is_some() {
               ctx.unregister_texture(self.texture_handle.take().unwrap());
           }
           if let Some(old_handle) = self.manual_texture_view_handle {
               manual_texture_views.remove(&old_handle);
               self.manual_texture_view_handle = None;
           }
           // Create new texture for camera render target
           let format = TextureFormat::Rgba8UnormSrgb;
           let wgpu_texture = self.wgpu_device.create_texture(&wgpu::TextureDescriptor {
               label: Some("bevy_graph_texture"),
               size: wgpu::Extent3d {
                   width,
                   height,
                   depth_or_array_layers: 1,
               },
               mip_level_count: 1,
               sample_count: 1,
               dimension: wgpu::TextureDimension::D2,
               format,
               usage: wgpu::TextureUsages::TEXTURE_BINDING
                   | wgpu::TextureUsages::RENDER_ATTACHMENT
                   | wgpu::TextureUsages::COPY_SRC,
               view_formats: &[],
           });

           let wgpu_texture_view =
               wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

           let manual_texture_view = ManualTextureView {
               texture_view: wgpu_texture_view.into(),
               size: bevy::math::UVec2::new(width, height),
               format,
           };

           let manual_texture_view_handle = ManualTextureViewHandle(0);
           manual_texture_views.insert(manual_texture_view_handle, manual_texture_view);
           // Update camera to render to this texture
           if let Ok(mut camera) = world.query::<&mut Camera>().single_mut(world) {
               camera.target = RenderTarget::TextureView(manual_texture_view_handle);
               self.last_texture_size = current_size;
               self.manual_texture_view_handle = Some(manual_texture_view_handle);
               self.texture_handle = Some(ctx.register_texture(wgpu_texture));
           }
       }
   }
}

File 4: `ui/src/knowledge_graph/graph_scene_plugin.rs`

This contains the Bevy systems for the 3D scene:

use bevy::prelude::*;
use std::collections::HashMap;
use crate::knowledge_graph::GraphMessage;
/// Resource containing graph data updated from Dioxus
#[derive(Resource, Default)]
pub struct GraphData {
   pub nodes: HashMap<String, NodeData>,
   pub edges: Vec<EdgeData>,
   pub highlighted_node: Option<String>,
}
#[derive(Clone)]
pub struct NodeData {
   pub id: String,
   pub position: Vec3,
   pub label: String,
}
#[derive(Clone)]
pub struct EdgeData {
   pub from: String,
   pub to: String,
}
impl GraphData {
   pub fn process_message(&mut self, msg: GraphMessage) {
       match msg {
           GraphMessage::AddNode { id, position, label } => {
               self.nodes.insert(id.clone(), NodeData {
                   id,
                   position: Vec3::from_array(position),
                   label,
               });
           }
           GraphMessage::AddEdge { from, to } => {
               self.edges.push(EdgeData { from, to });
           }
           GraphMessage::RemoveNode { id } => {
               self.nodes.remove(&id);
               self.edges.retain(|e| e.from != id && e.to != id);
           }
           GraphMessage::UpdateNodePosition { id, position } => {
               if let Some(node) = self.nodes.get_mut(&id) {
                   node.position = Vec3::from_array(position);
               }
           }
           GraphMessage::HighlightNode { id } => {
               self.highlighted_node = id;
           }
           GraphMessage::SetCameraPosition { .. } => {
               // Handle camera updates if needed
           }
       }
   }
}
/// Component marker for graph nodes
#[derive(Component)]
pub struct GraphNode {
   pub id: String,
}
/// Component marker for graph edges
#[derive(Component)]
pub struct GraphEdge {
   pub from: String,
   pub to: String,
}
pub struct GraphScenePlugin;
impl Plugin for GraphScenePlugin {
   fn build(&self, app: &mut App) {
       app.insert_resource(ClearColor(bevy::color::Color::srgba(0.0, 0.0, 0.0, 0.0)));
       app.add_systems(Startup, setup_scene);
       app.add_systems(Update, (
           spawn_nodes,
           spawn_edges,
           update_node_highlights,
           rotate_camera,
       ));
   }
}
fn setup_scene(mut commands: Commands) {
   // Setup camera
   commands.spawn((
       Camera3d::default(),
       Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
       Name::new("GraphCamera"),
   ));
   // Setup lighting
   commands.spawn((
       DirectionalLight {
           color: bevy::color::Color::WHITE,
           illuminance: 10000.0,
           shadows_enabled: false,
           ..default()
       },
       Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
   ));
   commands.insert_resource(AmbientLight {
       color: bevy::color::Color::WHITE,
       brightness: 200.0,
       affects_lightmapped_meshes: true,
   });
}
/// System to spawn new nodes that don't have entities yet
fn spawn_nodes(
   mut commands: Commands,
   graph_data: Res<GraphData>,
   existing_nodes: Query<&GraphNode>,
   mut meshes: ResMut<Assets<Mesh>>,
   mut materials: ResMut<Assets<StandardMaterial>>,
) {
   if !graph_data.is_changed() {
       return;
   }
   // Get IDs of existing nodes
   let existing_ids: std::collections::HashSet<_> =
       existing_nodes.iter().map(|n| n.id.clone()).collect();
   // Spawn new nodes
   for (id, node_data) in &graph_data.nodes {
       if !existing_ids.contains(id) {
           commands.spawn((
               Mesh3d(meshes.add(Sphere::new(0.3))),
               MeshMaterial3d(materials.add(StandardMaterial {
                   base_color: bevy::color::Color::srgb(0.3, 0.6, 0.9),
                   metallic: 0.5,
                   perceptual_roughness: 0.3,
                   ..default()
               })),
               Transform::from_translation(node_data.position),
               GraphNode { id: id.clone() },
               Name::new(format!("Node_{}", id)),
           ));
       }
   }
}
/// System to spawn edges between nodes
fn spawn_edges(
   mut commands: Commands,
   graph_data: Res<GraphData>,
   existing_edges: Query<&GraphEdge>,
   node_query: Query<(&GraphNode, &Transform)>,
   mut meshes: ResMut<Assets<Mesh>>,
   mut materials: ResMut<Assets<StandardMaterial>>,
) {
   if !graph_data.is_changed() {
       return;
   }
   // Get existing edges
   let existing: std::collections::HashSet<_> = existing_edges
       .iter()
       .map(|e| (e.from.clone(), e.to.clone()))
       .collect();
   // Create node position lookup
   let node_positions: HashMap<String, Vec3> = node_query
       .iter()
       .map(|(node, transform)| (node.id.clone(), transform.translation))
       .collect();
   // Spawn new edges
   for edge in &graph_data.edges {
       if !existing.contains(&(edge.from.clone(), edge.to.clone())) {
           if let (Some(&from_pos), Some(&to_pos)) =
               (node_positions.get(&edge.from), node_positions.get(&edge.to))
           {
               // Create a cylinder between the two nodes
               let direction = to_pos - from_pos;
               let length = direction.length();
               let midpoint = from_pos + direction * 0.5;

               // Calculate rotation to point cylinder from from_pos to to_pos
               let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
               commands.spawn((
                   Mesh3d(meshes.add(Cylinder::new(0.05, length))),
                   MeshMaterial3d(materials.add(StandardMaterial {
                       base_color: bevy::color::Color::srgb(0.7, 0.7, 0.7),
                       ..default()
                   })),
                   Transform::from_translation(midpoint).with_rotation(rotation),
                   GraphEdge {
                       from: edge.from.clone(),
                       to: edge.to.clone(),
                   },
                   Name::new(format!("Edge_{}_{}", edge.from, edge.to)),
               ));
           }
       }
   }
}
/// System to highlight selected nodes
fn update_node_highlights(
   graph_data: Res<GraphData>,
   node_query: Query<(&GraphNode, &MeshMaterial3d<StandardMaterial>)>,
   mut materials: ResMut<Assets<StandardMaterial>>,
) {
   if !graph_data.is_changed() {
       return;
   }
   for (node, mesh_material) in node_query.iter() {
       if let Some(material) = materials.get_mut(&mesh_material.0) {
           if Some(&node.id) == graph_data.highlighted_node.as_ref() {
               // Highlighted color
               material.base_color = bevy::color::Color::srgb(1.0, 0.8, 0.2);
               material.emissive = bevy::color::LinearRgba::new(1.0, 0.8, 0.2, 1.0);
           } else {
               // Normal color
               material.base_color = bevy::color::Color::srgb(0.3, 0.6, 0.9);
               material.emissive = bevy::color::LinearRgba::BLACK;
           }
       }
   }
}
/// Simple camera rotation for demo purposes
fn rotate_camera(
   time: Res<Time>,
   mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
   for mut transform in camera_query.iter_mut() {
       let rotation_speed = 0.2;
       let angle = time.elapsed_secs() * rotation_speed;
       let radius = 10.0;

       transform.translation.x = angle.cos() * radius;
       transform.translation.z = angle.sin() * radius;
       transform.look_at(Vec3::ZERO, Vec3::Y);
   }
}


─────────────────────────────────────────────────

Phase 3: Dioxus Component Integration

File 5: `ui/src/components/knowledge_graph_view.rs`

This is the Dioxus component that embeds the Bevy canvas:

use dioxus::prelude::*;
use dioxus_native::use_wgpu;
use crate::knowledge_graph::{GraphPaintSource, GraphMessage};
use schema::Note;
#[component]
pub fn KnowledgeGraphView(notes: Vec<Note>) -> Element {
   // Create the paint source and get a sender for communication
   let paint_source = GraphPaintSource::new();
   let sender = paint_source.sender();
   let paint_source_id = use_wgpu(move || paint_source);
   // Track which node is highlighted
   let mut highlighted_node = use_signal(|| None::<String>);
   // Initialize graph with notes when they change
   use_effect(move || {
       // Clear existing graph
       // (In production, you'd want to diff and only update changes)

       // Add nodes for each note
       for (i, note) in notes.iter().enumerate() {
           let angle = (i as f32) * std::f32::consts::TAU / notes.len() as f32;
           let radius = 5.0;
           let position = [
               angle.cos() * radius,
               0.0,
               angle.sin() * radius,
           ];
           sender.send(GraphMessage::AddNode {
               id: note.id_.clone().unwrap_or_default(),
               position,
               label: format!("Note {}", i),
           }).ok();
       }
       // Add edges between notes (example: connect sequential notes)
       for i in 0..notes.len().saturating_sub(1) {
           if let (Some(from_id), Some(to_id)) =
               (&notes[i].id_, &notes[i + 1].id_)
           {
               sender.send(GraphMessage::AddEdge {
                   from: from_id.clone(),
                   to: to_id.clone(),
               }).ok();
           }
       }
   });
   // Update highlighted node
   use_effect(move || {
       sender.send(GraphMessage::HighlightNode {
           id: highlighted_node(),
       }).ok();
   });
   rsx! {
       div {
           class: "knowledge-graph-container",
           style: "width: 100%; height: 100%; display: flex; flex-direction: column;",
           // Control panel
           div {
               class: "graph-controls",
               style: "padding: 20px; background: rgba(0,0,0,0.7); color: white;",

               h2 { "Knowledge Graph" }
               p { "Nodes: {notes.len()}" }

               button {
                   onclick: move |_| {
                       // Example: highlight first node
                       if let Some(note) = notes.first() {
                           highlighted_node.set(note.id_.clone());
                       }
                   },
                   "Highlight First Node"
               }
           }
           // 3D Canvas
           div {
               class: "graph-canvas-container",
               style: "flex: 1; position: relative;",

               canvas {
                   id: "knowledge-graph-canvas",
                   "src": paint_source_id,
                   style: "width: 100%; height: 100%;"
               }
           }
       }
   }
}


────────────────────────────────────────────

Phase 4: Integration with Neuramancer Routes

File 6: `web/src/views/graph.rs`

use dioxus::prelude::*;
use ui::components::KnowledgeGraphView;
#[component]
pub fn GraphView() -> Element {
   // Fetch notes from server
   let notes = use_server_future(move || backend::read_notes())?;
   rsx! {
       div {
           id: "graph-view",
           style: "width: 100vw; height: 100vh;",
           match notes() {
               Some(Ok(notes_data)) => rsx! {
                   KnowledgeGraphView { notes: notes_data }
               },
               Some(Err(e)) => rsx! {
                   div { "Error loading notes: {e}" }
               },
               None => rsx! {
                   div { "Loading graph..." }
               }
           }
       }
   }
}

Update `web/src/main.rs`

use views::{Blog, Home, NotFound, Notes, GraphView};  // Add GraphView
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
   #[layout(WebNavbar)]
   #[route("/")]
   Home {},
   #[route("/notes")]
   Notes {},
   #[route("/graph")]  // NEW: Graph route
   GraphView {},
   #[route("/blog/:id")]
   Blog { id: i32 },
   #[route("/:..segments")]
   NotFound { segments: Vec<String> },
}


───────────────────────────────────────

3. Knowledge Graph Specific Adaptations

Connecting to SurrealDB Data

Schema Extension: `schema/src/graph.rs`

use serde::{Deserialize, Serialize};
/// Graph-specific representation of a note
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
   pub id: String,
   pub label: String,
   pub note_type: String,
   pub position: Option<[f32; 3]>,  // Cached position
}
/// Relationship between notes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
   pub from: String,
   pub to: String,
   pub relationship_type: String,  // e.g., "references", "contains", "related_to"
}
impl From<&Note> for GraphNode {
   fn from(note: &Note) -> Self {
       GraphNode {
           id: note.id_.clone().unwrap_or_default(),
           label: format!("Note with {} snippets", note.snippets.len()),
           note_type: "note".to_string(),
           position: None,  // Will be calculated by layout algorithm
       }
   }
}

Bidirectional Communication Examples

Dioxus → Bevy: Creating a Note

#[component]
pub fn NoteCreator() -> Element {
   let graph_sender = use_context::<Sender<GraphMessage>>();

   let create_note = move |_| async move {
       // Create note in database
       let note = create_note_in_db("New note content").await?;

       // Add to 3D graph
       graph_sender.send(GraphMessage::AddNode {
           id: note.id_.unwrap(),
           position: [0.0, 0.0, 0.0],  // Will be positioned by layout
           label: "New Note".to_string(),
       }).ok();

       Ok::<(), ServerFnError>(())
   };
   rsx! {
       button { onclick: create_note, "Create Note" }
   }
}

Bevy → Dioxus: Node Click Detection

For click detection, you'd need to add a system in graph_scene_plugin.rs:

// Add to GraphScenePlugin systems
app.add_systems(Update, detect_node_clicks);
fn detect_node_clicks(
   // Note: This requires additional setup for mouse input in headless mode
   // You might need to pass mouse coordinates from Dioxus
   graph_data: Res<GraphData>,
   node_query: Query<(&GraphNode, &Transform)>,
) {
   // Raycast logic here
   // When node is clicked, update GraphData with selected node
   // Dioxus will read this on next frame
}


─────────────────────────────────────

4. Complete Code Examples

Main Entry Point (Desktop)

// ui/src/main.rs (for desktop app)
use dioxus::prelude::*;
fn main() {
   #[cfg(feature = "desktop")]
   {
       let config: Vec<Box<dyn std::any::Any>> = vec![
           Box::new(dioxus_native::Limits {
               max_storage_buffers_per_shader_stage: 12,
               ..Default::default()
           })
       ];
       dioxus_native::launch_cfg(app, Vec::new(), config);
   }
}
fn app() -> Element {
   rsx! {
       Router::<Route> {}
   }
}

Force-Directed Layout Algorithm

Add to graph_scene_plugin.rs:

/// System to apply force-directed layout
fn apply_force_layout(
   graph_data: Res<GraphData>,
   mut node_query: Query<(&GraphNode, &mut Transform)>,
   time: Res<Time>,
) {
   let dt = time.delta_secs();
   let repulsion_strength = 5.0;
   let attraction_strength = 0.1;
   let damping = 0.9;
   // Calculate forces
   let mut forces: HashMap<String, Vec3> = HashMap::new();
   // Repulsion between all nodes
   let nodes: Vec<_> = node_query.iter().collect();
   for i in 0..nodes.len() {
       for j in (i+1)..nodes.len() {
           let (node_a, transform_a) = &nodes[i];
           let (node_b, transform_b) = &nodes[j];

           let diff = transform_b.translation - transform_a.translation;
           let dist = diff.length().max(0.1);
           let force = diff.normalize() * (repulsion_strength / (dist * dist));
           *forces.entry(node_a.id.clone()).or_insert(Vec3::ZERO) -= force;
           *forces.entry(node_b.id.clone()).or_insert(Vec3::ZERO) += force;
       }
   }
   // Attraction along edges
   for edge in &graph_data.edges {
       if let (Some(from_transform), Some(to_transform)) = (
           node_query.iter().find(|(n, _)| n.id == edge.from).map(|(_, t)| t.translation),
           node_query.iter().find(|(n, _)| n.id == edge.to).map(|(_, t)| t.translation),
       ) {
           let diff = to_transform - from_transform;
           let force = diff * attraction_strength;
           *forces.entry(edge.from.clone()).or_insert(Vec3::ZERO) += force;
           *forces.entry(edge.to.clone()).or_insert(Vec3::ZERO) -= force;
       }
   }
   // Apply forces
   for (node, mut transform) in node_query.iter_mut() {
       if let Some(force) = forces.get(&node.id) {
           transform.translation += *force * dt * damping;
       }
   }
}


─────────────────────────────────────────────────────────────

5. Integration Points with Neuramancer

With Existing Schema

// In graph_paint_source.rs
impl From<&Note> for Vec<GraphMessage> {
   fn from(note: &Note) -> Self {
       let mut messages = vec![];

       // Add node for the note
       if let Some(id) = &note.id_ {
           messages.push(GraphMessage::AddNode {
               id: id.clone(),
               position: [0.0, 0.0, 0.0],
               label: format!("Note: {}", id),
           });
           // Add nodes for each snippet
           for snippet in &note.snippets {
               if let Some(snippet_id) = &snippet.id_ {
                   messages.push(GraphMessage::AddNode {
                       id: snippet_id.clone(),
                       position: [0.0, 0.0, 0.0],
                       label: format!("Snippet: {}", snippet_id),
                   });
                   // Add edge from note to snippet
                   messages.push(GraphMessage::AddEdge {
                       from: id.clone(),
                       to: snippet_id.clone(),
                   });
               }
           }
       }
       messages
   }
}

With SurrealDB Backend

// In backend/src/surreal/kv_mem.rs
pub async fn get_graph_data(&self) -> Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
   // Query all notes
   let notes: Vec<Note> = self.db
       .select("note")
       .await?;
   let mut nodes = vec![];
   let mut edges = vec![];
   for note in notes {
       nodes.push(GraphNode::from(&note));

       // Add edges based on note relationships
       // (You'd need to define these in your schema)
   }
   Ok((nodes, edges))
}

With Dioxus Fullstack SSR

Important Note: The example uses dioxus-native which is for desktop applications, not web. For web deployment with SSR, you have two options:

 1. Desktop-only graph view: Keep the 3D graph as a desktop-only feature
 2. WebGPU/WebGL fallback: Adapt the code to use Bevy's WASM support (more complex)

For web support, you'd need to:
 • Use Bevy's WASM features
 • Target a canvas element in the DOM
 • Handle the different initialization path


────────────────────────────────────────────

6. Key Patterns & Techniques

Pattern 1: Shared WGPU Device

The most critical pattern is reusing the WGPU device between Dioxus and Bevy:

// This is why it works without conflicts:
RenderCreation::Manual(RenderResources(
   RenderDevice::new(WgpuWrapper::new(device_handle.device.clone())),
   // ... other resources from Dioxus
))

Pattern 2: Message Passing via MPSC

Communication uses standard Rust channels:

// Dioxus side:
sender.send(GraphMessage::AddNode { ... }).ok();
// Bevy side (in render loop):
for msg in messages {
   graph_data.process_message(msg);
}

Pattern 3: Headless Bevy App

Bevy runs without a window:

.set(WindowPlugin {
   primary_window: None,  // No window
   exit_condition: bevy::window::ExitCondition::DontExit,
   close_when_requested: false,
   ..Default::default()
})
.disable::<bevy::winit::WinitPlugin>()  // No winit

Pattern 4: Texture-Based Rendering

Bevy renders to a texture that Dioxus displays:

// Bevy creates texture
let wgpu_texture = device.create_texture(...);
// Register with Dioxus
let texture_handle = ctx.register_texture(wgpu_texture);
// Dioxus displays it
canvas { "src": texture_handle }


───────────────────────────────────────

Summary

This integration pattern allows you to:

 1. ✅ Render complex 3D scenes with Bevy's powerful ECS
 2. ✅ Build UI with Dioxus's reactive components
 3. ✅ Share state bidirectionally via message passing
 4. ✅ Reuse WGPU resources efficiently
 5. ✅ Keep concerns separated (UI vs 3D rendering)

The key insight is that Bevy runs headless and renders to a texture that Dioxus displays in a canvas element, with communication happening
through MPSC channels and shared resources.

For your knowledge graph, this means you can have a rich 3D visualization of your notes and their relationships while maintaining a clean Dioxus
UI for editing and navigation.
