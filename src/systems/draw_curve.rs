use bevy_app::EventWriter;
use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::{
    geometry::curve::Curve,
    resources::{events::CurveChangedEvent, wall_manager::*},
    CursorRaycast,
};

use super::mode_manager::Mode;

pub fn draw_curve(
    _mode: Res<Mode>,

    mut ev_curve_changed: EventWriter<CurveChangedEvent>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,

    mouse_button_input: Res<Input<MouseButton>>,
) {
    if !matches!(*_mode, Mode::Wall) {
        return;
    }

    // Remove y component from the cursor-terrain raycast position
    let mut cursor_ws = cursor_ws.0;
    cursor_ws.y = 0.0;

    puffin::profile_function!();
    // If LMB was just pressed, start a new curve
    if mouse_button_input.just_pressed(MouseButton::Left) {
        wall_manager.temp_curve = Some(Curve::new());
        wall_manager.curves.push((Curve::new(), None));

        ev_curve_changed.send(CurveChangedEvent {
            curve_index: wall_manager.curves.len() - 1,
        });
    }
    // If LMB is pressed, continue the active curve
    else if mouse_button_input.pressed(MouseButton::Left) {
        let temp_curve = wall_manager.temp_curve.as_mut().unwrap();

        let intersection = cursor_ws;

        const DIST_THRESHOLD: f32 = 0.001;

        if temp_curve
            .points
            .last()
            // if curve  had points, only add if the distance is larger than X
            .map(|last| intersection.distance(*last) > DIST_THRESHOLD)
            // if curve  has no points, add this point
            .unwrap_or(true)
        {
            temp_curve.add(intersection);

            if temp_curve.points.len() > 2 {
                let clone_temp_curve = temp_curve.clone();
                let (active_curve, _) = wall_manager.curves.last_mut().unwrap();
                *active_curve = clone_temp_curve
                    .smooth(SMOOTHING_STEPS)
                    .resample(RESAMPLING);
            }

            ev_curve_changed.send(CurveChangedEvent {
                curve_index: wall_manager.curves.len() - 1,
            });
        }
    }
}
