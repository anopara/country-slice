use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::resources::{CurveSSBOCache, CurveSegmentsComputePass};

// pass curve ssbo data to compute_indirect
pub fn update_curve_ssbo(
    mouse_button_input: Res<Input<MouseButton>>,
    mut compute_indirect: ResMut<CurveSegmentsComputePass>,
    curve_data_cache: Res<CurveSSBOCache>,
) {
    puffin::profile_function!();
    // HACK: ideally, this should be an eveent that tells that curves have been update and SSBO needs updating
    if mouse_button_input.pressed(MouseButton::Left) {
        //log::debug!("Updating curves_buffer buffer...");
        compute_indirect.curves_buffer.update(&curve_data_cache.0);
    }
}
