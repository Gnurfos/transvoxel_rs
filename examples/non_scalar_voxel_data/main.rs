use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::prelude::Mesh as BevyMesh;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology::TriangleList;
use bevy::window::close_on_esc;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use transvoxel::mesh_builder::{GridPoint, MeshBuilder, VertexIndex};
use transvoxel::traits::{Density, VoxelData};
use transvoxel::transition_sides::no_side;

#[path = "../shared/flycam.rs"]
mod flycam;
use flycam::{FlyCamera, FlyCameraPlugin};
use transvoxel::voxel_source::Block;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mats_cache: ResMut<MaterialsResource>,
    mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,
    ambient_light: ResMut<AmbientLight>,
) {
    primary_window_query.single_mut().title = "Transvoxel example".to_string();
    load_materials(&mut materials, &mut mats_cache);
    spawn_light(&mut commands, ambient_light);
    spawn_camera(&mut commands);
    load_model(&mut commands, &mut meshes, &mats_cache);
}

fn load_materials(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mats_cache: &mut ResMut<MaterialsResource>,
) {
    mats_cache.solid_model = materials.add(Color::rgb(1.0, 1.0, 1.0).into());
}

fn spawn_light(_commands: &mut Commands, mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.color = Color::rgb(1.0, 1.0, 1.0);
    ambient_light.brightness = 1.0;
}

fn spawn_camera(commands: &mut Commands) {
    let cam_transform =
        Transform::from_xyz(30.0, 40.0, 30.0).looking_at(Vec3::new(5.0, 5.0, 5.0), Vec3::Y);
    let mut cam_bundle = commands.spawn(Camera3dBundle {
        transform: cam_transform,
        ..Default::default()
    });
    cam_bundle.insert(FlyCamera {
        enabled: true,
        mouse_motion_enabled: false,
        key_forward: KeyCode::Up,
        key_backward: KeyCode::Down,
        key_left: KeyCode::Left,
        key_right: KeyCode::Right,
        key_up: KeyCode::PageUp,
        key_down: KeyCode::PageDown,
        sensitivity: 9.0,
        ..Default::default()
    });
}

#[derive(Debug, Copy, Clone, Default)]
enum Material {
    #[default]
    Nothing,
    Soil,
    Grass,
    Metal,
}

impl Material {
    pub fn to_color(&self) -> Option<[f32; 4]> {
        match self {
            Material::Nothing => None,
            Material::Soil => Some([0.8, 0.7, 0.6, 1.0]),
            Material::Grass => Some([0.1, 0.8, 0.1, 1.0]),
            Material::Metal => Some([0.9, 0.9, 0.1, 1.0]),
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct CustomVoxelData {
    material: Material,
    density: f32,
}

impl VoxelData for CustomVoxelData {
    type Density = f32;

    fn density(&self) -> Self::Density {
        self.density
    }
}

#[derive(Default)]
struct CustomMeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub triangle_indices: Vec<usize>,
    vertices: usize,
}

impl CustomMeshBuilder {
    pub fn build(self) -> BevyMesh {
        let mut bevy_mesh = BevyMesh::new(TriangleList);
        let converted_indices: Vec<u32> = self.triangle_indices.iter().map(|i| *i as u32).collect();
        let indices = bevy::render::mesh::Indices::U32(converted_indices);
        bevy_mesh.set_indices(Some(indices));
        bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, self.positions);
        bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, self.normals);
        bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_COLOR, self.colors);
        return bevy_mesh;
    }
}

impl MeshBuilder<CustomVoxelData, f32> for CustomMeshBuilder {
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<CustomVoxelData, f32>,
        point_b: GridPoint<CustomVoxelData, f32>,
        interp_toward_b: <CustomVoxelData as VoxelData>::Density,
    ) -> VertexIndex {
        let position = point_a
            .position
            .interp_toward(&point_b.position, interp_toward_b);
        let gradient_x =
            point_a.gradient.0 + interp_toward_b * (point_b.gradient.0 - point_a.gradient.0);
        let gradient_y =
            point_a.gradient.1 + interp_toward_b * (point_b.gradient.1 - point_a.gradient.1);
        let gradient_z =
            point_a.gradient.2 + interp_toward_b * (point_b.gradient.2 - point_a.gradient.2);
        let normal = f32::gradients_to_normal(gradient_x, gradient_y, gradient_z);
        let color_a = point_a.voxel_data.material.to_color();
        let color_b = point_b.voxel_data.material.to_color();
        let color = match (color_a, color_b) {
            (None, None) => [0.0, 0.0, 0.0, 0.0], // Probably should not happen
            (None, Some(c)) => c, // At the limit something-nothing, we want to use the pure something color, and not blend it nor attenuate it
            (Some(c), None) => c, // Idem
            (Some(color_a), Some(color_b)) => [
                (color_a[0] + interp_toward_b * color_b[0]),
                (color_a[1] + interp_toward_b * color_b[1]),
                (color_a[2] + interp_toward_b * color_b[2]),
                (color_a[3] + interp_toward_b * color_b[3]),
            ],
        };
        self.positions.push([position.x, position.y, position.z]);
        self.normals.push(normal);
        self.colors.push(color);
        let index = self.vertices;
        self.vertices += 1;
        return VertexIndex(index);
    }

    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    ) {
        self.triangle_indices.push(vertex_1_index.0);
        self.triangle_indices.push(vertex_2_index.0);
        self.triangle_indices.push(vertex_3_index.0);
    }
}

fn field(x: f32, y: f32, z: f32) -> CustomVoxelData {
    // Main ground. Wavy
    let ground_level = 10f32 + 2.0 * (x / 2.0).sin();
    // let ground_level = 0.0;
    let grass_depth = 2.0;
    // Bubbles of yellow stuff are embedded at regular intervals in the ground
    let bubbles_period = 8.0;
    let bubble_radius = 2.0;
    let closest_bubble_center = (
        bubbles_period * (x / bubbles_period).round(),
        bubbles_period * (y / bubbles_period).round(),
        bubbles_period * (z / bubbles_period).round(),
    );
    let distance_to_bubble = distance(
        x,
        y,
        z,
        closest_bubble_center.0,
        closest_bubble_center.1,
        closest_bubble_center.2,
    );
    // A big crater in the middle
    let hole_radius = 10.0;
    let hole_center = (5.0, 10.0, 5.0);
    let distance_to_hole = distance(x, y, z, hole_center.0, hole_center.1, hole_center.2);
    // Computations
    let density = if distance_to_hole < hole_radius {
        0.0
    } else if y >= ground_level {
        0.0
    } else {
        1.0
    };
    let material = if distance_to_bubble < bubble_radius {
        Material::Metal
    } else if y < ground_level - grass_depth {
        Material::Soil
    } else if y < ground_level {
        Material::Grass
    } else {
        Material::Nothing
    };
    CustomVoxelData { material, density }
}

fn distance(x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2) + (z1 - z2) * (z1 - z2)).sqrt()
}

fn load_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    mats_cache: &ResMut<MaterialsResource>,
) {
    let subdivisions = 50;
    let block_size = 10f32;
    let threshold = 0f32;
    for dx in [-1, 0, 1] {
        for dy in [-1, 0, 1] {
            for dz in [-1, 0, 1] {
                let base = [
                    dx as f32 * block_size,
                    dy as f32 * block_size,
                    dz as f32 * block_size,
                ];
                let block = Block::from(base, block_size, subdivisions);
                let mesh_builder = transvoxel::extraction::extract_from_field(
                    field,
                    &block,
                    threshold,
                    no_side(),
                    CustomMeshBuilder::default(),
                );
                let bevy_mesh = mesh_builder.build();
                let mat = mats_cache.solid_model.clone();
                commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(bevy_mesh),
                        material: mat,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .insert(ModelMarkerComponent {});
            }
        }
    }
}

#[derive(Component)]
struct ModelMarkerComponent {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(bevy_screen_diags::ScreenDiagsPlugin::default())
        .add_startup_system(setup)
        .add_plugin(FlyCameraPlugin)
        .add_system(close_on_esc)
        .add_plugin(EguiPlugin)
        .init_resource::<MaterialsResource>()
        .add_system(ui)
        .add_system(clicks_handler)
        .run();
}

fn ui(mut _commands: Commands, mut egui_context: EguiContexts) {
    let win = egui::Window::new("Controls");
    win.show(egui_context.ctx_mut(), |ui| {
        if ui.button("Quit").clicked() {
            std::process::exit(0);
        }
        ui.label("Arrows/PgUp/PgDn to move the camera\nLeft-click/drag to rotate the camera\nEsc to quit");
    });
}

#[derive(Default, Resource)]
struct MaterialsResource {
    pub solid_model: Handle<StandardMaterial>,
}

fn clicks_handler(mut events: EventReader<MouseButtonInput>, mut cam_query: Query<&mut FlyCamera>) {
    for event in events.iter() {
        if event.button == MouseButton::Left {
            if event.state == ButtonState::Pressed {
                for mut cam in cam_query.iter_mut() {
                    cam.mouse_motion_enabled = true;
                }
            } else {
                for mut cam in cam_query.iter_mut() {
                    cam.mouse_motion_enabled = false;
                }
            }
        }
    }
}
