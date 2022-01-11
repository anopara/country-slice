use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

#[derive(Debug)]
pub enum Mode {
    Wall,
    Path,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Wall
    }
}

pub fn mode_manager(mut mode: ResMut<Mode>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Key1) {
        *mode = Mode::Wall;
    }

    if keys.just_pressed(KeyCode::Key2) {
        *mode = Mode::Path;
    }
}
