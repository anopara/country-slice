use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

use crate::asset_libraries::mesh_library::AssetMeshLibrary;

#[derive(Debug)]
pub enum Mode {
    Wall,
    Path,
    Erase,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Wall
    }
}

pub fn mode_manager(
    mut mode: ResMut<Mode>,
    keys: Res<Input<KeyCode>>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
) {
    if keys.just_pressed(KeyCode::Key1) {
        *mode = Mode::Wall;

        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([1.0, 1.0, 1.0]);
    }

    if keys.just_pressed(KeyCode::Key2) {
        *mode = Mode::Path;
        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([0.1, 0.7, 0.1]);
    }

    if keys.just_pressed(KeyCode::Key3) {
        *mode = Mode::Erase;
        let c = assets_mesh.get_handle_by_name("cube").unwrap();
        let m = assets_mesh.get_mut(c).unwrap();
        m.add_color([0.1, 0.0, 0.0]);
    }
}
