use {
    super::GraphInputState,
    bevy::{
        core_pipeline::tonemapping::Tonemapping, input::mouse::AccumulatedMouseScroll, prelude::*,
    },
    knowledge_space_core::{EdgeKind, GraphEdgeInput, GraphNodeInput, SpaceTier},
    std::{collections::HashMap, f32::consts::FRAC_PI_2},
};

pub struct KnowledgeSpacePlugin;

impl Plugin for KnowledgeSpacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphAppliedSnapshot::default())
            .insert_resource(InputState::default())
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (sync_graph_nodes, draw_graph_edges, draw_glyphs))
            .add_systems(
                Update,
                (
                    update_zoom,
                    update_rotation,
                    handle_tap_focus,
                    update_camera_transform,
                )
                    .chain(),
            );
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

#[derive(Component)]
struct SpaceEntity;

#[derive(Component)]
struct GlyphComponent {
    glyph: glyph_core::Glyph,
}

/// Orbit camera state stored as a component on the camera entity.
#[derive(Component)]
struct OrbitCamera {
    /// The point the camera orbits around / looks at.
    target: Vec3,
    /// Current distance from the target (smoothly interpolated).
    distance: f32,
    /// Desired distance from the target (set by scroll/pinch input).
    target_distance: f32,
    /// Vertical angle in radians (clamped to avoid flipping).
    pitch: f32,
    /// Horizontal angle in radians.
    yaw: f32,
    /// Minimum zoom distance.
    min_distance: f32,
    /// Maximum zoom distance.
    max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 12.0,
            target_distance: 12.0,
            pitch: 0.45,
            yaw: -0.3,
            min_distance: 2.0,
            max_distance: 50.0,
        }
    }
}

/// Tracks input state for camera controls.
#[derive(Resource, Default)]
struct InputState {
    /// Previous distance between two fingers (for pinch zoom).
    prev_pinch_distance: Option<f32>,
    /// Accumulated mouse-drag distance (px) since the last left-button press.
    left_drag_distance: f32,
}

fn setup_scene(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    // Configure gizmo rendering: render in front of meshes, wider lines.
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();

    config.depth_bias = -1.0;
    config.line.width = 5.0;

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
        OrbitCamera::default(),
        Transform::default(), // will be set by update_camera_transform
    ));

    // Debug: spawn a hardcoded bright red test glyph at the origin.
    commands.spawn((
        SpaceEntity,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlyphComponent {
            glyph: glyph_core::Glyph {
                strokes: vec![
                    glyph_core::Stroke {
                        points: vec![[0.0, 2.0, 0.0], [2.0, 0.0, 0.0]],
                        colour: [1.0, 0.0, 0.0],
                    },
                    glyph_core::Stroke {
                        points: vec![[2.0, 0.0, 0.0], [0.0, -2.0, 0.0]],
                        colour: [1.0, 0.0, 0.0],
                    },
                    glyph_core::Stroke {
                        points: vec![[0.0, -2.0, 0.0], [-2.0, 0.0, 0.0]],
                        colour: [1.0, 0.0, 0.0],
                    },
                    glyph_core::Stroke {
                        points: vec![[-2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
                        colour: [1.0, 0.0, 0.0],
                    },
                ],
            },
        },
    ));
}

fn sync_graph_nodes(
    mut commands: Commands,
    shared_state: Option<Res<GraphInputState>>,
    mut applied: ResMut<GraphAppliedSnapshot>,
    existing: Query<Entity, With<SpaceEntity>>,
) {
    let Some(shared_state) = shared_state
    else {
        eprintln!("sync_graph_nodes: no GraphInputState resource");
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
        eprintln!("sync_graph_nodes: no change, skipping");
        return;
    }

    let existing_count = existing.iter().count();

    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    for (i, node) in snapshot.nodes.iter().enumerate() {
        let position = Vec3::new(node.position[0], node.position[1], node.position[2]);
        let has_glyph = node.glyph.is_some();
        let mut entity_commands =
            commands.spawn((SpaceEntity, Transform::from_translation(position)));

        if let Some(glyph) = &node.glyph {
            entity_commands.insert(GlyphComponent {
                glyph: glyph.clone(),
            });
        }
    }

    applied.tier = snapshot.tier;
    applied.nodes = snapshot.nodes;
    applied.edges = snapshot.edges;
}

fn draw_glyphs(mut gizmos: Gizmos, query: Query<(&Transform, &GlyphComponent)>) {
    let count = query.iter().count();

    for (transform, glyph_component) in &query {
        let origin = transform.translation;

        for stroke in &glyph_component.glyph.strokes {
            let colour = Color::srgb(stroke.colour[0], stroke.colour[1], stroke.colour[2]);
            let world_points: Vec<Vec3> = stroke
                .points
                .iter()
                .map(|p| origin + Vec3::from_array(*p))
                .collect();

            // Use linestrip for all strokes to ensure continuous connected lines
            // This guarantees strokes connect properly since they're positioned end-to-end
            if world_points.len() >= 2 {
                gizmos.linestrip(world_points, colour);
            }
        }
    }
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

        let colour = match edge.kind {
            EdgeKind::NoteMembership => Color::srgb_u8(120, 168, 255),
            EdgeKind::KnotMembership => Color::srgb_u8(120, 210, 165),
            EdgeKind::ParentChild => Color::srgb_u8(252, 200, 114),
        };

        gizmos.line(*from, *to, colour);
    }
}

// ---------------------------------------------------------------------------
// Camera control systems
// ---------------------------------------------------------------------------

const SCROLL_ZOOM_SPEED: f32 = 0.02;
const PINCH_ZOOM_SPEED: f32 = 0.02;
/// Smoothing factor for zoom interpolation (higher = snappier, 0..∞).
const ZOOM_SMOOTHING: f32 = 8.0;
const MOUSE_ROTATE_SPEED: f32 = 0.005;
const TOUCH_ROTATE_SPEED: f32 = 0.005;
/// Maximum movement (in screen px) for a touch to count as a "tap" rather than a drag.
const TAP_MOVE_THRESHOLD: f32 = 10.0;

/// Zoom: mouse scroll wheel + two-finger pinch (computed from touch distance).
///
/// Uses multiplicative (logarithmic) scaling so the zoom feels uniform at every
/// distance.  Input sets `target_distance`; the actual `distance` is smoothly
/// interpolated toward it each frame by `smooth_zoom`.
fn update_zoom(
    mouse_scroll: Res<AccumulatedMouseScroll>,
    touches: Res<Touches>,
    mut input_state: ResMut<InputState>,
    time: Res<Time>,
    mut query: Query<&mut OrbitCamera>,
) {
    let mut zoom_factor: f32 = 0.0;

    // Mouse scroll wheel: negative y = scroll down = zoom out.
    if mouse_scroll.delta.y.abs() > 0.0 {
        // Multiplicative: each scroll notch scales distance by (1 ± speed).
        zoom_factor -= mouse_scroll.delta.y * SCROLL_ZOOM_SPEED;
    }

    // Two-finger pinch: compute distance between the first two pressed touches.
    let pressed: Vec<_> = touches.iter().collect();

    if pressed.len() >= 2 {
        let dist = pressed[0].position().distance(pressed[1].position());

        if let Some(prev) = input_state.prev_pinch_distance {
            let pinch_delta = prev - dist; // fingers moving apart → negative → zoom in
            zoom_factor += pinch_delta * PINCH_ZOOM_SPEED;
        }

        input_state.prev_pinch_distance = Some(dist);
    }
    else {
        input_state.prev_pinch_distance = None;
    }

    // Update target_distance with multiplicative zoom.
    if zoom_factor.abs() > f32::EPSILON {
        for mut cam in &mut query {
            let multiplier = 1.0 + zoom_factor;
            cam.target_distance =
                (cam.target_distance * multiplier).clamp(cam.min_distance, cam.max_distance);
        }
    }

    // Smoothly interpolate distance toward target_distance.
    let dt = time.delta_secs();
    let t = 1.0 - (-ZOOM_SMOOTHING * dt).exp(); // exponential ease-out

    for mut cam in &mut query {
        cam.distance = cam.distance + (cam.target_distance - cam.distance) * t;
    }
}

/// Rotation: left-mouse-button drag (desktop) or single-finger drag (touch).
///
/// Uses `CursorMoved` window events instead of `AccumulatedMouseMotion` because
/// the latter relies on `DeviceEvent::MouseMotion` which is not emitted on WASM.
fn update_rotation(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_moved: MessageReader<CursorMoved>,
    touches: Res<Touches>,
    mut input_state: ResMut<InputState>,
    mut query: Query<&mut OrbitCamera>,
) {
    let pitch_limit = FRAC_PI_2 - 0.01;
    let mut delta = Vec2::ZERO;

    // Accumulate cursor movement from CursorMoved window events.
    // On WASM, DeviceEvent::MouseMotion (which feeds AccumulatedMouseMotion)
    // does not fire, so we use CursorMoved.delta instead.
    let mut cursor_delta = Vec2::ZERO;
    let mut event_count = 0u32;
    for event in cursor_moved.read() {
        event_count += 1;
        if let Some(d) = event.delta {
            cursor_delta += d;
        }
    }

    // Reset drag distance on fresh press so tap detection works.
    if mouse_buttons.just_pressed(MouseButton::Left) {
        input_state.left_drag_distance = 0.0;
    }

    // Desktop: left-mouse-button drag.
    if mouse_buttons.pressed(MouseButton::Left) {
        input_state.left_drag_distance += cursor_delta.length();
        delta += cursor_delta * MOUSE_ROTATE_SPEED;
    }

    // Touch: single-finger drag (when exactly one finger is down).
    let pressed: Vec<_> = touches.iter().collect();

    if pressed.len() == 1 {
        let touch_delta = pressed[0].delta();

        if touch_delta.length_squared() > 0.0 {
            delta += touch_delta * TOUCH_ROTATE_SPEED;
        }
    }

    if delta.length_squared() < f32::EPSILON {
        return;
    }

    for mut cam in &mut query {
        cam.yaw -= delta.x;
        cam.pitch = (cam.pitch - delta.y).clamp(-pitch_limit, pitch_limit);
    }
}

/// Tap-to-focus: left-click release (desktop) or quick tap (touch) focuses on
/// the nearest `GlyphComponent` entity.  We find the glyph whose world
/// position is closest to the camera-to-click ray and set the orbit target.
fn handle_tap_focus(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    input_state: Res<InputState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    glyph_query: Query<&Transform, With<GlyphComponent>>,
    mut orbit_query: Query<&mut OrbitCamera>,
) {
    // Determine the screen-space tap/click position (if any).
    let tap_position: Option<Vec2> = if mouse_buttons.just_released(MouseButton::Left) &&
        input_state.left_drag_distance < TAP_MOVE_THRESHOLD
    {
        // Desktop left-click (released with minimal drag): use cursor position.
        windows.iter().next().and_then(|w| w.cursor_position())
    }
    else {
        // Touch: detect a quick tap (just released, minimal movement).
        touches.iter_just_released().find_map(|t| {
            if t.distance().length() < TAP_MOVE_THRESHOLD {
                Some(t.position())
            }
            else {
                None
            }
        })
    };

    let Some(screen_pos) = tap_position
    else {
        return;
    };

    // Cast a ray from the camera through the tap position.
    let Ok((camera, cam_global)) = camera_query.single()
    else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_global, screen_pos)
    else {
        return;
    };

    // Find the glyph entity whose position is closest to the ray.
    let mut best: Option<(f32, Vec3)> = None;

    for transform in &glyph_query {
        let pos = transform.translation;
        // Point-to-ray distance.
        let to_point = pos - ray.origin;
        let projected = to_point.dot(*ray.direction);

        if projected < 0.0 {
            continue; // behind the camera
        }

        let closest_on_ray = ray.origin + *ray.direction * projected;
        let dist = pos.distance(closest_on_ray);

        if best.map_or(true, |(d, _)| dist < d) {
            best = Some((dist, pos));
        }
    }

    if let Some((_dist, focus_pos)) = best {
        for mut cam in &mut orbit_query {
            cam.target = focus_pos;
        }
    }
}

/// Recompute the camera `Transform` from the orbit parameters every frame.
fn update_camera_transform(mut query: Query<(&OrbitCamera, &mut Transform)>) {
    for (cam, mut transform) in &mut query {
        transform.rotation = Quat::from_euler(EulerRot::YXZ, cam.yaw, cam.pitch, 0.0);
        transform.translation = cam.target - transform.forward() * cam.distance;
    }
}
