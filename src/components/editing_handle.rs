use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary};

use super::{Transform, TriggerArea};

#[derive(Bundle)]
pub struct EditingHandle {
    pub parent_curve: usize, // which curve does it belong
                             //pub trigger_area: TriggerArea,
                             //pub trigger_area_preview: TriggerAreaPreview,
}

impl EditingHandle {
    pub fn new(parent_curve: usize) -> Self {
        Self {
            parent_curve,
            //trigger_area: TriggerArea::new(20, Transform::from_translation(position)),
        }
    }

    /*
    pub fn add_world_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        self.trigger_area
            .add_world_space_preview(assets_mesh, assets_shader, commands)
    }

    pub fn add_screen_space_preview(
        &mut self,
        assets_mesh: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        self.trigger_area
            .add_screen_space_preview(assets_mesh, assets_shader, commands)
    }
    */
}
