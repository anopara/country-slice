use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use crate::resources::{events::CurveDeletedEvent, wall_manager::WallManager};

pub fn delete_curve(
    mut ev_curve_deleted: EventReader<CurveDeletedEvent>,
    mut wall_manager: ResMut<WallManager>,

    mut commands: Commands,
) {
    for ev in ev_curve_deleted.iter() {
        // Clear out preview entity if there is one
        if let Some(preview_ent) = wall_manager
            .walls
            .get_mut(&ev.curve_index)
            .unwrap()
            .curve_preview_entity
        {
            commands.entity(preview_ent).despawn();
        }
        // Remove the curve entry
        wall_manager.walls.remove_entry(&ev.curve_index);
    }
}
