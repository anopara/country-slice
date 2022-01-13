use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};
use glam::Vec3;

use crate::{components::CursorRaycast, geometry::curve::Curve, resources::WallManager};

use super::mode_manager::Mode;

pub fn erase_curve(
    _mode: Res<Mode>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if !matches!(*_mode, Mode::Erase) {
        return;
    }

    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    const ERASE_BRUSH_SIZE: f32 = 0.75;
    let cursor_ws = cursor_ws.0;

    // Go through all the curves
    let mut curves_to_replace = Vec::new();

    for (i, (curve, ent)) in wall_manager.curves.iter().enumerate() {
        let c: Vec<Vec3> = curve
            .points
            .iter()
            // Go through all the points
            .filter_map(|pt| {
                // Check if within distance
                if cursor_ws.distance(*pt) > ERASE_BRUSH_SIZE {
                    Some(*pt)
                } else {
                    // if so delete this point
                    None
                }
            })
            .collect();

        if c.len() != curve.points.len() {
            curves_to_replace.push((i, c, ent.clone()));
        }
    }

    const DIST_THRESHOLD: f32 = 0.1;
    for (i, mut c, ent) in curves_to_replace {
        // go through every curve, and see if the dist between a point >, then its a new curve and we need to split
        let mut splits = Vec::new();
        for (i, pt) in c.iter().enumerate() {
            if let Some(next) = c.get(i + 1) {
                if pt.distance(*next) > DIST_THRESHOLD {
                    splits.push(i);
                }
            }
        }

        // split
        let mut last_split_index = 0;
        let mut new_curves = Vec::new();
        for s in splits {
            new_curves.push(c.split_off(s - last_split_index));
            last_split_index = s;
        }
        new_curves.push(c);

        // Update curves
        wall_manager.curves[i] = (Curve::from(new_curves[0].clone()), ent);

        // send event that wall needs to be recalculated
    }
}
