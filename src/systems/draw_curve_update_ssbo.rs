use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::resources::{CurveSegmentsComputePass, WallManager};

// for compute indirect
pub fn update_curve_ssbo(
    wall_manager: Res<WallManager>,
    mouse_button_input: Res<Input<MouseButton>>,
    //mut compute_indirect: ResMut<ComputeArchesIndirect>,
    mut compute_indirect: ResMut<CurveSegmentsComputePass>,
) {
    puffin::profile_function!();
    // HACK: ideally, this should be an eveent that tells that curves have been update and SSBO needs updating
    if mouse_button_input.pressed(MouseButton::Left) {
        // HACK: update all the curves, bc its easier (in the future, no need to redo the whole buffer....)
        let data: Vec<_> = wall_manager
            .curves
            .iter()
            .map(|(curve, _)| {
                if curve.points.len() > 0 {
                    let c = crate::CurveDataSSBO::from(&curve.clone().smooth(50).resample(0.2));
                    //println!("curve has {} points", c.points_count);
                    c
                } else {
                    // add empty
                    crate::CurveDataSSBO {
                        points_count: 0,
                        pad0: 0,
                        pad1: 0,
                        pad2: 0,
                        positions: [[0.0; 4]; 1000],
                    }
                }
            })
            .collect();

        //log::debug!("Updating curves_buffer buffer...");
        compute_indirect.curves_buffer.update(&data);
    }
}
