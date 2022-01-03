use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};

use crate::{
    geometry::instanced_wall::InstancedWall,
    resources::{ComputePathsMask, CurveSegmentsComputePass, WallManager},
};

// Clear walls
// Clear curves
// Clear paths
// Clear shadow strips

pub fn clear_canvas(
    keys: Res<Input<KeyCode>>,
    mut wall_manager: ResMut<WallManager>,
    mut query_walls: Query<&mut InstancedWall>,
    compute_indirect: ResMut<CurveSegmentsComputePass>,
    mut compute_path_mask: ResMut<ComputePathsMask>,
    mut commands: Commands,
) {
    if keys.pressed(KeyCode::Back) {
        for wall_entity in &wall_manager.walls {
            let _wall_component = query_walls.get_mut(*wall_entity).unwrap();
            //wall_component.free_memory(); // TODO: I can make a single SSBO for all walls, then I don't need to manage individual memory chunks
            commands.entity(*wall_entity).despawn();
        }
        wall_manager.walls = Vec::new();

        for (_curve, maybe_curve_entity) in &wall_manager.curves {
            if let Some(curve_entity) = maybe_curve_entity {
                commands.entity(*curve_entity).despawn();
            }
        }
        wall_manager.curves = Vec::new();

        for shadow_entity in &wall_manager.shadows {
            commands.entity(*shadow_entity).despawn();
            // TODO: remove the mesh from Mesh library and VAO manager
        }
        wall_manager.shadows = Vec::new();

        // Clear our the path mask
        compute_path_mask.clear_texture();

        // Clear our the curve segments SSBO
        compute_indirect.reset_segments_buffer();
        compute_indirect.reset_cmd_buffer();
    }
}
