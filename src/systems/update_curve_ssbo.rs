use bevy_app::EventReader;
use bevy_ecs::prelude::*;
//use bevy_input::{mouse::MouseButton, Input};

use crate::resources::{
    events::CurveChangedEvent, CurveDataSSBO, CurveSegmentsComputePass, WallManager,
};

// pass curve ssbo data to compute_indirect
pub fn update_curve_ssbo(
    mut ev_curve_changed: EventReader<CurveChangedEvent>,
    wall_manager: Res<WallManager>,
    //mouse_button_input: Res<Input<MouseButton>>,
    mut compute_indirect: ResMut<CurveSegmentsComputePass>,
) {
    puffin::profile_function!();

    for ev in ev_curve_changed.iter() {
        let active_curve = &wall_manager.curves[ev.curve_index].0;

        let data = {
            puffin::profile_scope!("curve->SSBO");
            if active_curve.points.len() > 0 {
                CurveDataSSBO::from(&active_curve.clone().smooth(50).resample(0.2))
            } else {
                // add empty
                CurveDataSSBO {
                    points_count: 0,
                    pad0: 0,
                    pad1: 0,
                    pad2: 0,
                    positions: [[0.0; 4]; 1000],
                }
            }
        };

        puffin::profile_scope!("curve buffer update");
        compute_indirect
            .curves_buffer
            .update_element(data, wall_manager.curves.len() - 1);
    }

    /*
    // HACK: ideally, this should be an eveent that tells that curves have been update and SSBO needs updating
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some((active_curve, _)) = wall_manager.curves.last() {
            let data = {
                puffin::profile_scope!("curve->SSBO");
                if active_curve.points.len() > 0 {
                    CurveDataSSBO::from(&active_curve.clone().smooth(50).resample(0.2))
                } else {
                    // add empty
                    CurveDataSSBO {
                        points_count: 0,
                        pad0: 0,
                        pad1: 0,
                        pad2: 0,
                        positions: [[0.0; 4]; 1000],
                    }
                }
            };

            puffin::profile_scope!("curve buffer update");
            compute_indirect
                .curves_buffer
                .update_element(data, wall_manager.curves.len() - 1);
        }
    }
    */
}
