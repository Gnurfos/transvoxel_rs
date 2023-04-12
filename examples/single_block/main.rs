use std::f32::consts::PI;

use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::window::close_on_esc;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::render::mesh::Mesh as BevyMesh;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use transvoxel::structs::*;
use transvoxel::transition_sides::*;

#[path = "../shared/models.rs"]
mod models;
use models::Model;

#[path = "../shared/shapes.rs"]
mod shapes;
use shapes::create_arrow;

#[path = "../shared/utils.rs"]
mod utils;

#[path = "../shared/flycam.rs"]
mod flycam;
use flycam::{FlyCamera, FlyCameraPlugin};


const MAIN_BLOCK: BlockDims<f32> = BlockDims {
    base: [0.0, 0.0, 0.0],
    size: 10.0,
};

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mats_cache: ResMut<MaterialsResource>,
    mut primary_window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    primary_window_query.single_mut().title = "Transvoxel example".to_string();
    load_materials(&mut materials, mats_cache);
    spawn_background(&mut commands, &mut meshes, &mut materials);
    spawn_light(&mut commands);
    spawn_camera(&mut commands);
}

fn load_materials(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mut mats_cache: ResMut<MaterialsResource>,
) {
    mats_cache.solid_model = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    mats_cache.wireframe_model = materials.add(StandardMaterial {
        emissive: Color::BISQUE,
        unlit: true,
        ..Default::default()
    });
    mats_cache.grid = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: Color::rgb(0.6, 0.6, 0.6),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        ..Default::default()
    });
    mats_cache.grid_dot = mats_cache.grid.clone();
}

fn spawn_background(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(BevyMesh::from(shape::Plane {
            subdivisions: 4,
            size: MAIN_BLOCK.size,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(
            MAIN_BLOCK.size / 2.0 + MAIN_BLOCK.base[0],
            MAIN_BLOCK.base[1],
            MAIN_BLOCK.size / 2.0 + MAIN_BLOCK.base[2],
        ),
        ..Default::default()
    });
    // Axis X
    let arrow = create_arrow();
    let arrow_handle = meshes.add(arrow);
    commands.spawn(PbrBundle {
        mesh: arrow_handle.clone(),
        material: materials.add(Color::rgb(0.9, 0.2, 0.2).into()),
        transform: Transform::from_xyz(0.5, 0.0, 0.0),
        ..Default::default()
    });
    // Axis Y
    commands.spawn(PbrBundle {
        mesh: arrow_handle.clone(),
        material: materials.add(Color::rgb(0.2, 0.9, 0.2).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0)
            * Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
        ..Default::default()
    });
    // Axis Z
    commands.spawn(PbrBundle {
        mesh: arrow_handle.clone(),
        material: materials.add(Color::rgb(0.2, 0.2, 0.9).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.5)
            * Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0)),
        ..Default::default()
    });
}

fn spawn_light(commands: &mut Commands) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        point_light: PointLight {
            range: 100.0,
            radius: 250.0,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn spawn_camera(commands: &mut Commands) {
    let cam_transform =
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::new(5.0, 5.0, 5.0), Vec3::Y);
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

fn add_initial_model(mut channel: EventWriter<AppEvent>) {
    channel.send(AppEvent::LoadModel);
}

fn load_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    mats_cache: &Res<MaterialsResource>,
    model_params: &ModelParams,
) {
    let wireframe = model_params.wireframe;
    let transition_sides = if model_params.with_transition {
        TransitionSide::LowX.into()
    } else {
        no_side()
    };
    let block = Block {
        dims: MAIN_BLOCK,
        subdivisions: model_params.subdivisions,
    };
    let bevy_mesh = utils::mesh_for_model(&model_params.model, wireframe, &block, &transition_sides);
    let mat = if wireframe {
        mats_cache.wireframe_model.clone()
    } else {
        mats_cache.solid_model.clone()
    };
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(bevy_mesh),
            material: mat,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(ModelMarkerComponent {});
    if model_params.show_grid {
        add_grid(
            commands,
            meshes,
            mats_cache,
            model_params,
            &transition_sides,
        );
    }
}

fn add_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    mats_cache: &Res<MaterialsResource>,
    model_params: &ModelParams,
    transition_sides: &TransitionSides,
) {
    let block = Block {
        dims: MAIN_BLOCK,
        subdivisions: model_params.subdivisions,
    };
    let grid_mesh = utils::grid_lines(&block, &transition_sides);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(grid_mesh),
            material: mats_cache.grid.clone(),
            ..Default::default()
        })
        .insert(ModelMarkerComponent {});
    let cube = BevyMesh::from(shape::Cube { size: 1.0 });
    let cube_handle = meshes.add(cube);
    for (x, y, z) in utils::inside_grid_points(&model_params.model, &block, &transition_sides) {
        let cell_size = MAIN_BLOCK.size / model_params.subdivisions as f32;
        let point_size = cell_size * 0.05;
        let resize = Transform::from_scale(Vec3::new(point_size, point_size, point_size));
        let rotate = Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            45f32.to_radians(),
            45f32.to_radians(),
            0.0,
        ));
        let translate = Transform::from_xyz(x, y, z);
        commands
            .spawn(PbrBundle {
                mesh: cube_handle.clone(),
                material: mats_cache.grid_dot.clone(),
                transform: translate * rotate * resize,
                ..Default::default()
            })
            .insert(ModelMarkerComponent {});
    }
}

#[derive(Component)]
struct ModelMarkerComponent {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(bevy_screen_diags::ScreenDiagsPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(add_initial_model)
        .add_plugin(FlyCameraPlugin)
        .add_system(close_on_esc)
        .add_plugin(EguiPlugin)
        .init_resource::<UiState>()
        .init_resource::<MaterialsResource>()
        .add_system(ui)
        .add_event::<AppEvent>()
        .add_system(app_events_handler)
        .add_system(clicks_handler)
        .run();
}

fn ui(
    mut _commands: Commands,
    mut egui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut channel: EventWriter<AppEvent>,
) {
    let mut model_changed = false;
    let win = egui::Window::new("Controls");
    win.show(egui_context.ctx_mut(), |ui| {
        if ui.button("Quit").clicked() {
            channel.send(AppEvent::Quit);
        }
        ui.label("Arrows/PgUp/PgDn to move the camera\nLeft-click/drag to rotate the camera\nEsc to quit");
        let text = format!("Change model (current: {:?})", ui_state.desired_things.model);
        let mut selected_model = ui_state.get_model();
        egui::ComboBox::from_label(text)
            .selected_text(format!("{:?}", selected_model))
            .show_ui(ui, |ui| {
                for m in models::Model::iterator() {
                    ui.selectable_value(&mut selected_model, *m, format!("{:?}", m));
                }
            });
        model_changed |= ui_state.set_model(selected_model);
        if ui.checkbox(&mut ui_state.desired_things.wireframe, "Wireframe").clicked() {
            model_changed = true;
        }
        if ui.checkbox(&mut ui_state.desired_things.show_grid, "Show grid (slow)").clicked() {
            model_changed = true;
        }
        const MAX_SUB: usize = 30;
        if ui.add(
            egui::Slider::new(&mut ui_state.desired_things.subdivisions, 1..=MAX_SUB).text("Subdivisions").clamp_to_range(true)
        ).changed() {
            model_changed = true;
        }
        ui.horizontal(|ui| {
            if ui.button("Less").clicked() {
                ui_state.desired_things.subdivisions = (ui_state.desired_things.subdivisions - 1).max(1);
                model_changed = true;
            }
            if ui.button("More").clicked() {
                ui_state.desired_things.subdivisions = (ui_state.desired_things.subdivisions + 1).min(MAX_SUB);
                model_changed = true;
            }
        });
        if ui.checkbox(&mut ui_state.desired_things.with_transition, "With transition").clicked() {
            model_changed = true;
        }

        if model_changed {
            channel.send(AppEvent::LoadModel);
        }
    });
}

#[derive(Default, Resource)]
struct UiState {
    pub desired_things: ModelParams,
}

struct ModelParams {
    pub model: Model,
    pub wireframe: bool,
    pub subdivisions: usize,
    pub show_grid: bool,
    pub with_transition: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            model: Model::Sphere,
            wireframe: false,
            subdivisions: 6,
            show_grid: false,
            with_transition: false,
        }
    }
}

impl UiState {
    pub fn set_model(&mut self, model: Model) -> bool {
        let changed = self.desired_things.model != model;
        self.desired_things.model = model;
        return changed;
    }
    pub fn get_model(&self) -> Model {
        self.desired_things.model
    }
}

#[derive(Default, Resource)]
struct MaterialsResource {
    pub solid_model: Handle<StandardMaterial>,
    pub wireframe_model: Handle<StandardMaterial>,
    pub grid: Handle<StandardMaterial>,
    pub grid_dot: Handle<StandardMaterial>,
}

enum AppEvent {
    LoadModel,
    Quit,
}

fn app_events_handler(
    mut events: EventReader<AppEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mats_cache: Res<MaterialsResource>,
    models_query: Query<(Entity, &ModelMarkerComponent)>,
    ui_state: Res<UiState>,
) {
    for event in events.iter() {
        match event {
            AppEvent::LoadModel => {
                let params = &ui_state.desired_things;
                for (entity, _) in models_query.iter() {
                    commands.entity(entity).despawn();
                }
                load_model(&mut commands, &mut meshes, &mats_cache, &params);
            }
            AppEvent::Quit => {
                std::process::exit(0);
            }
        }
    }
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
