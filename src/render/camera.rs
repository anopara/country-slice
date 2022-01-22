use dolly::prelude::*;
use glam::{Mat4, Vec3};

use crate::components::transform::Transform;

const CAMERA_FOV_DEGREES: f32 = 45.0;

pub struct MainCamera {
    pub camera: Camera,
    pub camera_rig: CameraRig,
}

impl MainCamera {
    pub fn new(aspect_ratio: f32) -> Self {
        let camera_rig = CameraRig::builder()
            .with(YawPitch::new().yaw_degrees(45.0).pitch_degrees(-30.0))
            .with(Position::new(Vec3::ZERO))
            //.with(Rotation::new(Quat::from_rotation_y(45.0_f32.to_radians()))) //
            //.with(YawPitch::new().yaw_degrees(0.0).pitch_degrees(90.0))
            .with(Smooth::new_position_rotation(1.0, 1.0))
            .with(Arm::new(dolly::glam::Vec3::Z * 9.0))
            //.with(Smooth::new_position(3.0))
            //.with(Arm::new(dolly::glam::Vec3::Z * 20.0))
            .build();
        let camera = Camera::new(
            Transform::identity(),
            CAMERA_FOV_DEGREES.to_radians(),
            aspect_ratio,
        );
        Self { camera, camera_rig }
    }
}

pub struct Camera {
    pub transform: Mat4,
    pub perspective_projection: Mat4, // TODO: hm... it really doesnt need to be a part of the camera
}

impl Camera {
    pub fn new(transform: Transform, fov: f32, aspect_ratio: f32) -> Self {
        // convert between glam 13.0 and glam 18.0
        let transform = Mat4::from_cols_array(&transform.compute_matrix().to_cols_array());

        Self {
            transform,
            perspective_projection: Mat4::perspective_rh_gl(fov, aspect_ratio, 0.1, 100.0),
        }
    }

    pub fn world_to_camera_view(&self) -> Mat4 {
        self.transform.inverse()
    }

    pub fn position(&self) -> Vec3 {
        let (_, _, t) = self.transform.to_scale_rotation_translation();
        t
    }
}
