use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::{
    components::{CursorRaycast, Transform},
    resources::wall_manager::WallManager,
};

use super::draw_wall::CONTINUE_CURVE_DIST_THRESHOLD;

// Component
pub struct SignfierContinueWall;

pub fn signifier_continue_wall(
    mut query: Query<(&SignfierContinueWall, &mut Transform)>,
    wall_manager: Res<WallManager>,
    cursor_ws: Res<CursorRaycast>,
) {
    let cursor_ws = cursor_ws.0;

    let mut signifier_pos = None;
    for (_, curve) in wall_manager.walls.iter().map(|(i, w)| (i, &w.curve)) {
        if let Some(last_pt) = curve.points.last() {
            if cursor_ws.distance(*last_pt) < CONTINUE_CURVE_DIST_THRESHOLD {
                if let Some(prev_point) = curve.points.get(curve.points.len() - 2) {
                    let dir = (*last_pt - *prev_point).normalize();
                    signifier_pos = Some(*last_pt + dir * 0.12);
                }

                break;
            }
        }

        if let Some(first_pt) = curve.points.get(0) {
            if cursor_ws.distance(*first_pt) < CONTINUE_CURVE_DIST_THRESHOLD {
                if let Some(next_pt) = curve.points.get(1) {
                    let dir = (*first_pt - *next_pt).normalize();
                    signifier_pos = Some(*first_pt + dir * 0.12);
                }

                break;
            }
        }
    }

    let (_, mut transform) = query.single_mut().unwrap();
    if let Some(signifier_pos) = signifier_pos {
        transform.translation = signifier_pos + Vec3::Y * 0.01;
    } else {
        transform.translation = Vec3::Y * -1.0;
    }
}
