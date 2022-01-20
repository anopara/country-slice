use bevy_app::EventWriter;
use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::{
    geometry::curve::Curve,
    resources::{events::CurveChangedEvent, wall_manager::*},
    systems::mode_manager::BrushMode,
    CursorRaycast,
};

pub fn draw_wall(
    _mode: Res<BrushMode>,

    mut ev_curve_changed: EventWriter<CurveChangedEvent>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,

    mouse_button_input: Res<Input<MouseButton>>,
) {
    if !matches!(*_mode, BrushMode::Wall) {
        return;
    }

    // Remove y component from the cursor-terrain raycast position
    let mut cursor_ws = cursor_ws.0;
    cursor_ws.y = 0.0;

    puffin::profile_function!();
    // If LMB was just pressed, start a new curve
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Check if we started next to existing curve, then just continue that curve!
        const CONTINUE_CURVE_DIST_THRESHOLD: f32 = 0.2;

        let mut continue_curve = None;
        for (idx, curve) in wall_manager.walls.iter().map(|(i, w)| (i, &w.curve)) {
            if let Some(last_pt) = curve.points.last() {
                if cursor_ws.distance(*last_pt) < CONTINUE_CURVE_DIST_THRESHOLD {
                    continue_curve = Some((*idx, curve.clone(), AddPointsTo::End));
                    break;
                }
            }

            if let Some(first_pt) = curve.points.get(0) {
                if cursor_ws.distance(*first_pt) < CONTINUE_CURVE_DIST_THRESHOLD {
                    continue_curve = Some((*idx, curve.clone(), AddPointsTo::Beginning));
                    break;
                }
            }
        }

        let index = if let Some((idx, curve, mode)) = continue_curve {
            let temp_curve = curve.resample(0.05); // resample to prevent deflation of the curve, when we start drawing
            wall_manager.temp_curve = Some(InProgressCurve::new(temp_curve, idx, mode));
            idx
        } else {
            wall_manager.new_wall(Curve::new())
        };

        ev_curve_changed.send(CurveChangedEvent { curve_index: index });
    }
    // If LMB is pressed, continue the active curve
    else if mouse_button_input.pressed(MouseButton::Left) {
        let temp_curve = wall_manager.temp_curve.as_mut().unwrap();

        let draw_mode = &temp_curve.mode;
        let active_curve_index = temp_curve.index;
        let active_curve = &mut temp_curve.curve;

        let intersection = cursor_ws;

        let active_curve_pt = match draw_mode {
            AddPointsTo::End => active_curve.points.len() - 1,
            AddPointsTo::Beginning => 0,
        };

        const DIST_THRESHOLD: f32 = 0.001;

        if active_curve
            .points
            .get(active_curve_pt)
            // if curve  had points, only add if the distance is larger than X
            .map(|pt| intersection.distance(*pt) > DIST_THRESHOLD)
            // if curve  has no points, add this point
            .unwrap_or(true)
        {
            match draw_mode {
                AddPointsTo::End => active_curve.add(intersection),
                AddPointsTo::Beginning => active_curve.add_to_front(intersection),
            }

            if active_curve.points.len() > 2 {
                let clone_temp_curve = active_curve.clone();
                wall_manager.get_mut(active_curve_index).unwrap().curve = clone_temp_curve
                    .smooth(SMOOTHING_STEPS)
                    .resample(RESAMPLING);
            }

            ev_curve_changed.send(CurveChangedEvent {
                curve_index: active_curve_index,
            });
        }
    }
}
