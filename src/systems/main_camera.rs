use bevy_core::Time;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};
use dolly::prelude::YawPitch;
use glam::Mat4;

use crate::render::camera::MainCamera;

pub fn main_camera_update(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut main_camera: ResMut<MainCamera>, //mut q: Query<(&mut Camera, &mut CameraRig)>,
) {
    let camera_driver = main_camera.camera_rig.driver_mut::<YawPitch>();

    let dt = time.delta_seconds();
    let speed_mult = 40.0;

    if keys.pressed(KeyCode::Left) {
        camera_driver.rotate_yaw_pitch(-2.0 * dt * speed_mult, 0.0);
    }
    if keys.pressed(KeyCode::Right) {
        camera_driver.rotate_yaw_pitch(2.0 * dt * speed_mult, 0.0);
    }

    if keys.pressed(KeyCode::Up) {
        camera_driver.rotate_yaw_pitch(0.0, -1.0 * dt * speed_mult);
    }
    if keys.pressed(KeyCode::Down) {
        camera_driver.rotate_yaw_pitch(0.0, 1.0 * dt * speed_mult);
    }

    let (pos, rot) = main_camera
        .camera_rig
        .update(time.delta_seconds())
        .into_position_rotation();
    main_camera.camera.transform = Mat4::from_rotation_translation(rot, pos);
}
