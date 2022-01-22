use bevy_app::EventReader;
use bevy_ecs::prelude::*;
//use bevy_input::{mouse::MouseButton, Input};

use crate::resources::{
    curve_segments_pass::CURVE_BUFFER_SIZE,
    events::{CurveChangedEvent, CurveDeletedEvent},
    CurveDataSSBO, CurveSegmentsComputePass, WallManager,
};

// pass curve ssbo data to compute_indirect
pub fn update_curve_ssbo(
    mut ev_curve_changed: EventReader<CurveChangedEvent>,
    mut ev_curve_deleted: EventReader<CurveDeletedEvent>,
    wall_manager: Res<WallManager>,
    //mouse_button_input: Res<Input<MouseButton>>,
    mut compute_indirect: ResMut<CurveSegmentsComputePass>,
) {
    puffin::profile_function!();

    for ev in ev_curve_changed.iter() {
        if ev.curve_index >= CURVE_BUFFER_SIZE {
            // TODO: atm curve index = SSBO layout index, however CURVE_BUFFER_SIZE is only allocated for 1000 elememnts,
            // and we keep adding to the end of the buffer without ever checking if new spots became available
            // Need to do some kind of manager, that makes sure to re-use indices that were freed if the curve was deleted
            panic!("Curve index is > CURVE_BUFFER_SIZE");
        }

        let active_curve = &wall_manager.get(ev.curve_index).unwrap().curve;

        let data = {
            puffin::profile_scope!("curve->SSBO");
            if active_curve.points.len() > 0 {
                CurveDataSSBO::from(&active_curve)
            } else {
                CurveDataSSBO::empty()
            }
        };

        puffin::profile_scope!("curve buffer update");
        compute_indirect
            .curves_buffer
            .update_element(data, ev.curve_index);
    }

    for ev in ev_curve_deleted.iter() {
        compute_indirect
            .curves_buffer
            .update_element(CurveDataSSBO::empty(), ev.curve_index);
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
