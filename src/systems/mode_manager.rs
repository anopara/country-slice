use bevy_app::EventWriter;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

use crate::resources::events::BrushModeJustChanged;

#[derive(Debug)]
pub enum EraseLayer {
    All,
    _Wall,
    _Path,
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
    mut ev_mode_changed: EventWriter<BrushModeJustChanged>,
    keys: Res<Input<KeyCode>>,
    //mut assets_mesh: ResMut<AssetMeshLibrary>,
) {
    if keys.just_pressed(KeyCode::Key1) {
        *mode = BrushMode::Wall;
        ev_mode_changed.send(BrushModeJustChanged {
            to: BrushMode::Wall,
        });
    }

    if keys.just_pressed(KeyCode::Key2) {
        *mode = BrushMode::Path;
        ev_mode_changed.send(BrushModeJustChanged {
            to: BrushMode::Path,
        });
    }

    if keys.just_pressed(KeyCode::Key3) {
        *mode = BrushMode::Eraser(EraseLayer::All);
        ev_mode_changed.send(BrushModeJustChanged {
            to: BrushMode::Eraser(EraseLayer::All),
        });
    }
}
