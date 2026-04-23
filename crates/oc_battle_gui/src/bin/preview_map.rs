use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

#[derive(Component)]
struct TerrainMesh;

// ── Your data source ──────────────────────────────────────────────────────────

/// Replace with your real data.
/// Returns (points, width, height) — row-major, points[row * width + col].
/// x = world X, y = height, z = world Z.
fn world_points() -> (Vec<Vec3>, usize, usize) {
    let (w, h) = (64usize, 64usize);
    let pts = (0..h)
        .flat_map(|row| {
            (0..w).map(move |col| {
                let x = col as f32;
                let z = row as f32;
                let y = ((x * 0.3).sin() + (z * 0.3).cos()) * 2.0;
                Vec3::new(x, y, z)
            })
        })
        .collect();
    (pts, w, h)
}

// ── App ───────────────────────────────────────────────────────────────────────

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (points, width, height) = world_points();
    let terrain_mesh = build_terrain_mesh(points, width, height);
    let mesh_handle: Handle<Mesh> = meshes.add(terrain_mesh);

    // Optional texture — place your PNG at assets/textures/terrain.png
    let texture_handle: Handle<Image> = asset_server.load("worlds_/minidblue/background.png");

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        })),
        TerrainMesh,
    ));

    // Centre the camera above the terrain
    let centre = Vec3::new(width as f32 * 0.5, 0.0, height as f32 * 0.5);
    let eye = centre
        + Vec3::new(
            0.0,
            width.max(height) as f32,
            width.max(height) as f32 * 0.6,
        );

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(eye).looking_at(centre, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 4_000_000.0,
            range: 1000.0,
            ..default()
        },
        Transform::from_translation(eye),
    ));

    commands.spawn((
        Text::new("Controls:\nX/Y/Z: Rotate  R: Reset"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// ── Input ─────────────────────────────────────────────────────────────────────

fn input_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<TerrainMesh>>,
    time: Res<Time>,
) {
    for mut transform in &mut query {
        if keyboard_input.pressed(KeyCode::KeyX) {
            transform.rotate_x(time.delta_secs() / 1.2);
        }
        if keyboard_input.pressed(KeyCode::KeyY) {
            transform.rotate_y(time.delta_secs() / 1.2);
        }
        if keyboard_input.pressed(KeyCode::KeyZ) {
            transform.rotate_z(time.delta_secs() / 1.2);
        }
        if keyboard_input.pressed(KeyCode::KeyR) {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

// ── Terrain mesh builder ──────────────────────────────────────────────────────

/// Build a Bevy `Mesh` from a flat row-major grid of `Vec3` world points.
///
/// - Smooth normals are accumulated from surrounding face normals.
/// - UVs map [col → u, row → v] linearly in [0, 1].
fn build_terrain_mesh(points: Vec<Vec3>, width: usize, height: usize) -> Mesh {
    assert_eq!(
        points.len(),
        width * height,
        "points.len() ({}) must equal width({}) * height({})",
        points.len(),
        width,
        height
    );

    let vert_count = points.len();

    // ── Positions ─────────────────────────────────────────────────────
    let positions: Vec<[f32; 3]> = points.iter().map(|p| [p.x, p.y, p.z]).collect();

    // ── UVs — stretch texture over the whole grid ──────────────────────
    let uvs: Vec<[f32; 2]> = (0..height)
        .flat_map(|row| {
            (0..width).map(move |col| {
                [
                    col as f32 / (width - 1).max(1) as f32,
                    row as f32 / (height - 1).max(1) as f32,
                ]
            })
        })
        .collect();

    // ── Indices — two CCW triangles per quad ───────────────────────────
    let mut indices: Vec<u32> = Vec::with_capacity((width - 1) * (height - 1) * 6);
    for row in 0..height - 1 {
        for col in 0..width - 1 {
            let tl = (row * width + col) as u32;
            let tr = (row * width + col + 1) as u32;
            let bl = ((row + 1) * width + col) as u32;
            let br = ((row + 1) * width + col + 1) as u32;
            // Triangle A
            indices.extend_from_slice(&[tl, bl, tr]);
            // Triangle B
            indices.extend_from_slice(&[tr, bl, br]);
        }
    }

    // ── Smooth normals — accumulate face normals per vertex ────────────
    let mut normals: Vec<Vec3> = vec![Vec3::ZERO; vert_count];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = points[a];
        let pb = points[b];
        let pc = points[c];
        let face_normal = (pb - pa).cross(pc - pa); // area-weighted, not normalised
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }
    let normals: Vec<[f32; 3]> = normals
        .iter()
        .map(|n| {
            let n = n.normalize_or_zero();
            [n.x, n.y, n.z]
        })
        .collect();

    // ── Assemble Mesh ─────────────────────────────────────────────────
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}
