use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

use crate::asset_libraries::mesh_library::AssetMeshLibrary;

#[derive(Debug)]
pub enum EraseLayer {
    All,
}

#[derive(Debug)]
pub enum BrushMode {
    Wall,
    Path,
    Eraser(EraseLayer),
}

impl Default for BrushMode {
    fn default() -> Self {
        BrushMode::Wall
    }
}

pub fn mode_manager(
    mut mode: ResMut<BrushMode>,
    keys: Res<Input<KeyCode>>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
) {
    if keys.just_pressed(KeyCode::Key1) {
        *mode = BrushMode::Wall;

        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([1.0, 1.0, 1.0]);
    }

    if keys.just_pressed(KeyCode::Key2) {
        *mode = BrushMode::Path;
        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([0.1, 0.7, 0.1]);
    }

    if keys.just_pressed(KeyCode::Key3) {
        *mode = BrushMode::Eraser(EraseLayer::All);
        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([0.1, 0.0, 0.0]);
    }
}
