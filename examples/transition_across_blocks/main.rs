use std::f32::consts::PI;

use bevy::color::palettes::css;
use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_egui::egui::SliderClamping::Always;
use transvoxel::transition_sides::*;
use transvoxel::voxel_source::{Block, BlockDims};

#[path = "../shared/models.rs"]
mod models;
use models::Model;

#[path = "../shared/bevy_mesh.rs"]
mod bevy_mesh;

#[path = "../shared/shapes.rs"]
mod shapes;
use shapes::create_arrow;

#[path = "../shared/utils.rs"]
mod utils;

#[path = "../shared/flycam.rs"]
mod flycam;
use flycam::{FlyCamera, FlyCameraPlugin};

fn blocks_to_show(
    base_subdivisions: usize,
    with_transitions: bool,
) -> [(Block<f32>, flagset::FlagSet<TransitionSide>); 3] {
    [
        (
            Block {
                dims: BlockDims {
                    base: [0.0, 0.0, 0.0],
                    size: 10.0,
                },
                subdivisions: base_subdivisions,
            },
            if with_transitions {
                TransitionSide::LowX | TransitionSide::LowZ
            } else {
                no_side()
            },
        ),
        (
            Block {
                dims: BlockDims {
                    base: [-10.0, 0.0, 0.0],
                    size: 10.0,
                },
                subdivisions: base_subdivisions * 2,
            },
            no_side(),
        ),
        (
            Block {
                dims: BlockDims {
                    base: [0.0, 0.0, -10.0],
                    size: 10.0,
                },
                subdivisions: base_subdivisions * 2,
            },
            no_side(),
        ),
    ]
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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
    mats_cache.solid_model = materials.add(Color::srgb(0.8, 0.7, 0.6));
    mats_cache.wireframe_model = materials.add(StandardMaterial {
        emissive: css::BISQUE.into(),
        unlit: true,
        ..Default::default()
    });
    mats_cache.grid = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: Color::srgb(0.6, 0.6, 0.6).into(),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        ..Default::default()
    });
    mats_cache.grid_dot = mats_cache.grid.clone();
}

fn spawn_background(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Axis X
    let arrow = create_arrow();
    let arrow_handle = meshes.add(arrow);
    commands.spawn((
        Mesh3d(arrow_handle.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.2, 0.2))),
        Transform::from_xyz(0.5, 0.0, 0.0),
    ));

    // Axis Y
    commands.spawn((
        Mesh3d(arrow_handle.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.9, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0)
            * Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
    ));

    // Axis Z
    commands.spawn((
        Mesh3d(arrow_handle.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.9))),
        Transform::from_xyz(0.0, 0.0, 0.5)
            * Transform::from_rotation(Quat::from_rotation_z(-PI / 2.0)),
    ));
}

fn spawn_light(commands: &mut Commands) {
    commands.spawn((
        Transform::from_xyz(10.0, 10.0, 10.0),
        PointLight {
            range: 100.0,
            radius: 250.0,
            ..Default::default()
        },
    ));
}


fn spawn_camera(commands: &mut Commands) {
        let cam_transform =
            Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::new(5.0, 5.0, 5.0), Vec3::Y);

        let mut cam_bundle = commands.spawn((Camera3d::default(), cam_transform));
        cam_bundle.insert(FlyCamera {
            enabled: true,
            mouse_motion_enabled: false,
            key_forward: KeyCode::ArrowUp,
            key_backward: KeyCode::ArrowDown,
            key_left: KeyCode::ArrowLeft,
            key_right: KeyCode::ArrowRight,
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
        meshes: &mut ResMut<Assets<Mesh>>,
        mats_cache: &Res<MaterialsResource>,
        model_params: &ModelParams,
    ) {
        let wireframe = model_params.wireframe;
        for (block, transition_sides) in
            &blocks_to_show(model_params.subdivisions, model_params.with_transitions)
        {
            let bevy_mesh =
                utils::mesh_for_model(&model_params.model, wireframe, block, transition_sides);
            let mat = if wireframe {
                mats_cache.wireframe_model.clone()
            } else {
                mats_cache.solid_model.clone()
            };
            commands
                .spawn((
                    Mesh3d(meshes.add(bevy_mesh)),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ))
                .insert(ModelMarkerComponent {});
            if model_params.show_grid {
                add_grid(
                    commands,
                    meshes,
                    mats_cache,
                    model_params,
                    block,
                    transition_sides,
                );
            }
        }
    }

    fn add_grid(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        mats_cache: &Res<MaterialsResource>,
        model_params: &ModelParams,
        block: &Block<f32>,
        transition_sides: &TransitionSides,
    ) {
        let grid_mesh = utils::grid_lines(block, transition_sides);
        commands
            .spawn((
                Mesh3d(meshes.add(grid_mesh)),
                MeshMaterial3d(mats_cache.grid.clone()),
            ))
            .insert(ModelMarkerComponent {});
        let cube = Mesh::from(Cuboid::from_length(1.0));
        let cube_handle = meshes.add(cube);
        for (x, y, z) in utils::inside_grid_points(&model_params.model, block, transition_sides) {
            let cell_size = block.dims.size / block.subdivisions as f32;
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
                .spawn((
                    Mesh3d(cube_handle.clone()),
                    MeshMaterial3d(mats_cache.grid_dot.clone()),
                    translate * rotate * resize,
                ))
                .insert(ModelMarkerComponent {});
        }
    }

    #[derive(Component)]
    struct ModelMarkerComponent {}

    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins((FlyCameraPlugin, EguiPlugin))
            //.add_plugin(bevy_screen_diags::ScreenDiagsPlugin::default())
            .add_systems(Startup, (setup, add_initial_model))
            .init_resource::<UiState>()
            .init_resource::<MaterialsResource>()
            .add_event::<AppEvent>()
            .add_systems(Update, (ui, app_events_handler, clicks_handler))
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
                egui::Slider::new(&mut ui_state.desired_things.subdivisions, 1..=MAX_SUB).text("Subdivisions").clamping(Always)
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
            if ui.checkbox(&mut ui_state.desired_things.with_transitions, "With transition").clicked() {
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
        pub with_transitions: bool,
    }

    impl Default for ModelParams {
        fn default() -> Self {
            Self {
                model: Model::Quadrant,
                wireframe: false,
                subdivisions: 4,
                show_grid: false,
                with_transitions: true,
            }
        }
    }

    impl UiState {
        pub fn set_model(&mut self, model: Model) -> bool {
            let changed = self.desired_things.model != model;
            self.desired_things.model = model;
            changed
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

    #[derive(Event)]
    enum AppEvent {
        LoadModel,
        Quit,
    }

    fn app_events_handler(
        mut events: EventReader<AppEvent>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mats_cache: Res<MaterialsResource>,
        models_query: Query<(Entity, &ModelMarkerComponent)>,
        ui_state: Res<UiState>,
    ) {
        for event in events.read() {
            match event {
                AppEvent::LoadModel => {
                    let params = &ui_state.desired_things;
                    for (entity, _) in models_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    load_model(&mut commands, &mut meshes, &mats_cache, params);
                }
                AppEvent::Quit => {
                    std::process::exit(0);
                }
            }
        }
    }

    fn clicks_handler(mut events: EventReader<MouseButtonInput>, mut cam_query: Query<&mut FlyCamera>) {
        for event in events.read() {
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

