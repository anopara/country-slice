use glam::Vec3;

use crate::{asset_libraries::Asset, geometry::cube::Cube, render::mesh::Mesh};

pub struct UiPrompt {
    pub is_mouse_over: bool,
    pub padding: usize, // padding of the interaction bounding box in screen-space in pixels
    pub debug_preview: bevy_ecs::prelude::Entity,
}

impl UiPrompt {
    pub fn mesh_asset() -> Asset<Mesh> {
        Asset::new(Mesh::from(Cube::new(0.1))).name("ui_prompt")
    }
}

pub struct UiPromptDebugPreview; //2D line strip to draw to show the bounding box in screenspace

impl UiPromptDebugPreview {
    pub fn mesh_asset() -> Asset<Mesh> {
        // TODO: make this into a component? and has a custom draw for it in the render loop?
        let mut _preview = Mesh::new();
        _preview.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
        );
        _preview.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 0.0, 0.0]; 4]);
        _preview.set_indices(vec![0, 1, 2, 3]);

        Asset::new(_preview).name("ui_debug")
    }
}
