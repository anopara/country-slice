use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, mouse::MouseButton, Input};
use glam::Vec2;

use crate::TerrainData;

pub fn update_terrain(mut terrain: ResMut<TerrainData>, keys: Res<Input<KeyCode>>) {
    puffin::profile_function!();

    if keys.pressed(KeyCode::Space) {
        terrain.offset += Vec2::new(0.06, 0.06);
        terrain.recalculate_texture();
    }

    if keys.pressed(KeyCode::Q) {
        terrain.amp += 0.03;
        terrain.recalculate_texture();
    }

    if keys.pressed(KeyCode::E) {
        terrain.amp -= 0.03;
        terrain.recalculate_texture();
    }
}
