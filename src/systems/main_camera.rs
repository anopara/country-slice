use bevy_app::EventReader;
use bevy_core::Time;
use bevy_ecs::prelude::*;
use bevy_input::{
    mouse::{MouseButton, MouseWheel},
    Input,
};
use dolly::prelude::{Arm, Position, YawPitch};
use glam::Mat4;

use crate::{render::camera::MainCamera, window_events::CursorMoved};

pub fn main_camera_update(
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_wheel_ev: EventReader<MouseWheel>,
    mut cursor: EventReader<CursorMoved>,

    time: Res<Time>,
    mut main_camera: ResMut<MainCamera>,
) {
    let rot_speed_mult = 0.25;
    let pos_speed_mult = 0.02;
    let zoom_speed_mult = 0.5;

    if mouse_button_input.pressed(MouseButton::Right) {
        if let Some(cursor_latest) = cursor.iter().last() {
            let delta = cursor_latest.delta;
            main_camera
                .camera_rig
                .driver_mut::<YawPitch>()
                .rotate_yaw_pitch(1.2 * delta.x * rot_speed_mult, delta.y * rot_speed_mult);
        }
    }

    if mouse_button_input.pressed(MouseButton::Middle) {
        if let Some(cursor_latest) = cursor.iter().last() {
            let camera_transform = main_camera.camera.transform;

            // TODO: modulate how much we pan based on how much we are zoomed in

            let delta = cursor_latest.delta;
            let local_delta = glam::Vec3::new(delta.x, 0.0, delta.y);
            let mut ws_delta = camera_transform.transform_vector3(local_delta);

            // remove Y component TODO:: renormalize
            // TODO: or, instead, when making a camera rig, make a parent transform that rotates in Yaw, but pitch is separate
            // TODO: maybe also can be a use case for bevy to insert "checkpoints" of transforms one might want to extract?
            ws_delta.y = 0.0;

            main_camera
                .camera_rig
                .driver_mut::<Position>()
                .translate(ws_delta * pos_speed_mult);
        }
    }

    if let Some(mouse_wheel) = mouse_wheel_ev.iter().last() {
        // TODO: longer the wheel is used, it should get exp
        // TODO: add smoothness that only affects the offset of the arm but not the parent stuff, that gets nauseous! (or smoothing that only applies in one axis)
        // TODO: fork bevy and add ConstranedSmooth? that you can specify the axis of smoothing and ChildSmoothing, which only applies it to children?
        if mouse_wheel.y.abs() > 0.0 {
            main_camera.camera_rig.driver_mut::<Arm>().offset +=
                dolly::glam::Vec3::Z * mouse_wheel.y * zoom_speed_mult;
        }
    }

    let (pos, rot) = main_camera
        .camera_rig
        .update(time.delta_seconds())
        .into_position_rotation();
    main_camera.camera.transform = Mat4::from_rotation_translation(rot, pos);
}
