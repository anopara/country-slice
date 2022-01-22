use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use crate::{
    components::{CursorRaycast, Transform},
    resources::events::BrushModeJustChanged,
    systems::mode_manager::BrushMode,
};

// Component
#[derive(PartialEq, Eq)]
pub enum BrushPreview {
    Wall,
    Path,
    Eraser,
}

pub fn brush_preview(
    query: Query<(Entity, &BrushPreview)>,
    mut ev_mode_changed: EventReader<BrushModeJustChanged>,
    cursor_ws_cache: Res<CursorRaycast>,
    mut commands: Commands,
) {
    if let Some(BrushModeJustChanged { to }) = ev_mode_changed.iter().last() {
        let keep = match to {
            BrushMode::Wall => BrushPreview::Wall,
            BrushMode::Path => BrushPreview::Path,
            BrushMode::Eraser(_) => BrushPreview::Eraser,
        };

        for (ent, brush) in query.iter() {
            if *brush == keep {
                commands
                    .entity(ent)
                    .insert(Transform::from_translation(cursor_ws_cache.0));
            } else {
                // without Transform, the mesh will not render and its FollowMouse component will not be updated either
                commands.entity(ent).remove::<Transform>();
            }
        }
    }
}
