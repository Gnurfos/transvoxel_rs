
use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
pub struct FlyCamera {
	pub accel: f32,
	pub max_speed: f32,
	pub sensitivity: f32,
	pub friction: f32,
	pub yaw_pitch: Option<(f32, f32)>,
	pub velocity: Vec3,
	pub key_forward: KeyCode,
	pub key_backward: KeyCode,
	pub key_left: KeyCode,
	pub key_right: KeyCode,
	pub key_up: KeyCode,
	pub key_down: KeyCode,
	pub enabled: bool,
	pub mouse_motion_enabled: bool,
}
impl Default for FlyCamera {
	fn default() -> Self {
		Self {
			accel: 1.5,
			max_speed: 0.5,
			sensitivity: 3.0,
			friction: 1.0,
			yaw_pitch: None,
			velocity: Vec3::ZERO,
			key_forward: KeyCode::W,
			key_backward: KeyCode::S,
			key_left: KeyCode::A,
			key_right: KeyCode::D,
			key_up: KeyCode::Space,
			key_down: KeyCode::LShift,
			enabled: true,
			mouse_motion_enabled: true,
		}
	}
}

fn forward_vector(rotation: &Quat) -> Vec3 {
	rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
	let f = forward_vector(rotation);
	let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
	f_flattened
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
	Quat::from_rotation_y(90.0f32.to_radians())
		.mul_vec3(forward_walk_vector(rotation))
		.normalize()
}

fn camera_movement_system(
	time: Res<Time>,
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
	for (mut options, mut transform) in query.iter_mut() {
		let (axis_h, axis_v, axis_float) = if options.enabled {
			(
				movement_axis(&keyboard_input, options.key_right, options.key_left),
				movement_axis(
					&keyboard_input,
					options.key_backward,
					options.key_forward,
				),
				movement_axis(&keyboard_input, options.key_up, options.key_down),
			)
		} else {
			(0.0, 0.0, 0.0)
		};

		let rotation = transform.rotation;
		let accel: Vec3 = (strafe_vector(&rotation) * axis_h)
			+ (forward_walk_vector(&rotation) * axis_v)
			+ (Vec3::Y * axis_float);
		let accel: Vec3 = if accel.length() != 0.0 {
			accel.normalize() * options.accel
		} else {
			Vec3::ZERO
		};

		let friction: Vec3 = if options.velocity.length() != 0.0 {
			options.velocity.normalize() * -1.0 * options.friction
		} else {
			Vec3::ZERO
		};

		options.velocity += accel * time.delta_seconds();

		// clamp within max speed
		if options.velocity.length() > options.max_speed {
			options.velocity = options.velocity.normalize() * options.max_speed;
		}

		let delta_friction = friction * time.delta_seconds();

		options.velocity = if (options.velocity + delta_friction).signum()
			!= options.velocity.signum()
		{
			Vec3::ZERO
		} else {
			options.velocity + delta_friction
		};

		transform.translation += options.velocity;
	}
}

fn mouse_motion_system(
	time: Res<Time>,
	mut mouse_motion_event_reader: EventReader<MouseMotion>,
	mut query: Query<(&mut FlyCamera, &mut Transform)>,
) {
	let mut delta: Vec2 = Vec2::ZERO;
	for event in mouse_motion_event_reader.iter() {
		delta += event.delta;
	}
	if delta.is_nan() {
		return;
	}

	for (mut options, mut transform) in query.iter_mut() {
		if !options.enabled || !options.mouse_motion_enabled {
			continue;
		}
		let (previous_yaw, previous_pitch) = match options.yaw_pitch {
			None => get_yaw_pitch(&transform.rotation),
			Some(yaw_pitch) => yaw_pitch,
		};
		let new_yaw = previous_yaw - delta.x * options.sensitivity * time.delta_seconds();
		let mut new_pitch = previous_pitch + delta.y * options.sensitivity * time.delta_seconds();
		new_pitch = new_pitch.clamp(-89.0, 89.9);
		options.yaw_pitch = Some((new_yaw, new_pitch));
		let yaw_radians = new_yaw.to_radians();
		let pitch_radians = new_pitch.to_radians();
		transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
			* Quat::from_axis_angle(-Vec3::X, pitch_radians);
	}
}

fn get_yaw_pitch(rotation: &Quat) -> (f32, f32) {
	let q = rotation;
	let sinp = 2.0 * (q.w * q.x - q.y * q.z);
	let pitch = - sinp.asin();
    let siny_cosp = 2.0 * (q.w * q.y + q.z * q.x);
    let cosy_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
    let yaw = siny_cosp.atan2(cosy_cosp);
	(yaw.to_degrees(), pitch.to_degrees())
}

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(camera_movement_system)
			.add_system(mouse_motion_system);
	}
}

pub fn movement_axis(
	input: &Res<Input<KeyCode>>,
	plus: KeyCode,
	minus: KeyCode,
) -> f32 {
	let mut axis = 0.0;
	if input.pressed(plus) {
		axis += 1.0;
	}
	if input.pressed(minus) {
		axis -= 1.0;
	}
	axis
}
