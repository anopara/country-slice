use bevy_app::EventReader;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, mouse::MouseButton, Input};
use dolly::prelude::{Position, YawPitch};
use glam::Mat4;

use crate::{render::camera::MainCamera, window_events::CursorMoved};

pub fn main_camera_update(
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor: EventReader<CursorMoved>,

    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut main_camera: ResMut<MainCamera>, //mut q: Query<(&mut Camera, &mut CameraRig)>,
) {
    let time_dt = time.delta_seconds();
    let rot_speed_mult = 90.0;
    let pos_speed_mult = 10.0;

    if mouse_button_input.pressed(MouseButton::Right) {
        if let Some(cursor_latest) = cursor.iter().last() {
            let delta = cursor_latest.delta;
            main_camera
                .camera_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(
                    1.2 * delta.x * time_dt * rot_speed_mult,
                    delta.y * time_dt * rot_speed_mult,
                );
        }
    }

    if mouse_button_input.pressed(MouseButton::Middle) {
        if let Some(cursor_latest) = cursor.iter().last() {
            let delta = cursor_latest.delta;
            main_camera
                .camera_rig
                .driver_mut::<Position>()
                .translate(delta.extend(0.0) * time_dt * pos_speed_mult);
        }
    }

    /*
    if keys.pressed(KeyCode::Left) {
        camera_driver_rot.rotate_yaw_pitch(-2.0 * time_dt * rot_speed_mult, 0.0);
    }
    if keys.pressed(KeyCode::Right) {
        camera_driver_rot.rotate_yaw_pitch(2.0 * time_dt * rot_speed_mult, 0.0);
    }

    if keys.pressed(KeyCode::Up) {
        camera_driver_rot.rotate_yaw_pitch(0.0, -1.0 * time_dt * rot_speed_mult);
    }
    if keys.pressed(KeyCode::Down) {
        camera_driver_rot.rotate_yaw_pitch(0.0, 1.0 * time_dt * rot_speed_mult);
    }
    */

    let (pos, rot) = main_camera
        .camera_rig
        .update(time.delta_seconds())
        .into_position_rotation();
    main_camera.camera.transform = Mat4::from_rotation_translation(rot, pos);
}
