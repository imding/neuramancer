use {
    bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*},
    std::{f32::consts::FRAC_PI_2, sync::{Arc, Mutex}},
};

pub struct KnowledgeSpacePlugin;

impl Plugin for KnowledgeSpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene)
            .add_systems(Update, sync_note_cubes);
    }
}

#[derive(Resource, Clone)]
pub struct KnowledgeSpaceState {
    pub note_count: Arc<Mutex<usize>>,
}

#[derive(Resource, Clone)]
struct CubeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Clone, Copy, Debug)]
struct NoteCube {
    index: usize,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_assets = CubeAssets {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
    };

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    ));

    commands.insert_resource(cube_assets);

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
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn sync_note_cubes(
    mut commands: Commands,
    shared_state: Option<Res<KnowledgeSpaceState>>,
    cube_assets: Res<CubeAssets>,
    cubes: Query<(Entity, &NoteCube)>,
) {
    let Some(shared_state) = shared_state else {
        return;
    };

    let target_count = match shared_state.note_count.lock() {
        Ok(count) => *count,
        Err(poisoned) => *poisoned.into_inner(),
    };

    let mut existing: Vec<(Entity, usize)> = cubes
        .iter()
        .map(|(entity, cube)| (entity, cube.index))
        .collect();

    let current_count = existing.len();

    if target_count == current_count {
        return;
    }

    if target_count < current_count {
        existing.sort_by_key(|(_, index)| *index);
        for (entity, index) in existing.into_iter() {
            if index >= target_count {
                commands.entity(entity).despawn();
            }
        }
        return;
    }

    let start_index = current_count;
    for index in start_index..target_count {
        let position = index_to_position(index);
        commands.spawn((
            NoteCube { index },
            Mesh3d(cube_assets.mesh.clone()),
            MeshMaterial3d(cube_assets.material.clone()),
            Transform::from_xyz(position.x, position.y, position.z),
        ));
    }
}

fn index_to_position(index: usize) -> Vec3 {
    let per_row = 8;
    let spacing = 1.6;
    let row = index / per_row;
    let col = index % per_row;
    let x = col as f32 * spacing - (per_row as f32 - 1.0) * spacing * 0.5;
    let z = row as f32 * spacing;
    Vec3::new(x, 0.5, z)
}
