use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[allow(dead_code)]
impl Transform {
    #[inline]
    pub const fn identity() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    #[inline]
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Transform {
            translation,
            ..Default::default()
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Transform {
            scale,
            ..Default::default()
        }
    }

    pub fn from_translation_scale(translation: Vec3, scale: Vec3) -> Self {
        Transform {
            translation,
            scale,
            ..Default::default()
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
