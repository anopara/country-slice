use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use crate::resources::{events::CurveDeletedEvent, wall_manager::WallManager};

pub fn delete_wall(
    mut ev_curve_deleted: EventReader<CurveDeletedEvent>,
    mut wall_manager: ResMut<WallManager>,

    mut commands: Commands,
) {
    for ev in ev_curve_deleted.iter() {
        log::debug!("Wall index {} entry has been removed", ev.curve_index);
        wall_manager.remove_entry(ev.curve_index, &mut commands);
    }
}
