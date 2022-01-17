use bevy_app::EventWriter;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

use crate::resources::{
    events::CurveDeletedEvent, ComputePathMask, CurveSegmentsComputePass, WallManager,
};

// Clear walls
// Clear curves
// Clear paths
// Clear shadow strips

pub fn clear_canvas(
    keys: Res<Input<KeyCode>>,
    wall_manager: Res<WallManager>,
    mut ev_curve_deleted: EventWriter<CurveDeletedEvent>,
    compute_indirect: ResMut<CurveSegmentsComputePass>,
    mut compute_path_mask: ResMut<ComputePathMask>,
) {
    if keys.pressed(KeyCode::Back) {
        for (k, _) in &wall_manager.walls {
            ev_curve_deleted.send(CurveDeletedEvent { curve_index: *k });
        }

        // Clear our the path mask
        compute_path_mask.texture.clear();

        // Clear our the curve segments SSBO
        compute_indirect.reset_segments_buffer();
        compute_indirect.reset_cmd_buffer();
    }
}
