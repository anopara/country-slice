use crate::{
    components::CursorRaycast,
    geometry::curve::Curve,
    resources::{
        events::{CurveChangedEvent, CurveDeletedEvent},
        WallManager,
    },
};
use bevy_app::EventWriter;
use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};
use glam::{Vec2, Vec3};

use super::mode_manager::BrushMode;

pub fn eraser(
    _mode: Res<BrushMode>,
    mut ev_curve_changed: EventWriter<CurveChangedEvent>,
    mut ev_curve_deleted: EventWriter<CurveDeletedEvent>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    if !matches!(*_mode, BrushMode::Eraser(..)) {
        return;
    }

    if !mouse_button_input.pressed(MouseButton::Left) {
        return;
    }

    puffin::profile_function!();

    const ERASE_BRUSH_SIZE: f32 = 0.75;
    let cursor_ws = cursor_ws.0;

    let mut g_cc = Vec::new(); //new curves
    for (curve_index, wall) in wall_manager.walls.iter_mut() {
        let mut new_curves = vec![Vec::new()];
        let mut new_curve_last_index = 0;

        let curve = &wall.curve;
        for j in 0..(curve.points.len() - 1) {
            // segment
            let p1 = curve.points[j];
            let p2 = curve.points[j + 1];

            // TODO: intersect not just with the latest mouse position withi radius, but an interpolation from the previous frame
            let p1_is_inside = cursor_ws.distance(p1) < ERASE_BRUSH_SIZE;
            let p2_is_inside = cursor_ws.distance(p2) < ERASE_BRUSH_SIZE;

            match (p1_is_inside, p2_is_inside) {
                (false, false) => new_curves[new_curve_last_index].push(p1),
                (false, true) => {
                    new_curves[new_curve_last_index].push(p1);

                    let intersection = circle_segment_intersection(
                        vec3_xz(p1),
                        vec3_xz(p2),
                        vec3_xz(cursor_ws),
                        ERASE_BRUSH_SIZE,
                    );
                    new_curves[new_curve_last_index].push(vec2_x0y(intersection));

                    // this is the end of the curve outside the brush stroke
                    new_curves.push(Vec::new());
                    new_curve_last_index += 1;
                }
                (true, false) => {
                    // this is the beginning of the curve outside the brush stroke
                    let intersection = circle_segment_intersection(
                        vec3_xz(p1),
                        vec3_xz(p2),
                        vec3_xz(cursor_ws),
                        ERASE_BRUSH_SIZE,
                    );
                    new_curves[new_curve_last_index].push(vec2_x0y(intersection));
                }
                (true, true) => {} // delete segments that are fully inside
            }

            // if its the last segment
            if j == curve.points.len() - 2 && !p2_is_inside {
                new_curves[new_curve_last_index].push(p2);
            }
        }

        // check if no degenerate curves
        let mut cc = Vec::new();
        for n in new_curves {
            let c = Curve::from(n);
            if c.length > 0.0 {
                cc.push(c);
            }
        }

        // if no curves left, send an evene to delete this curve completely
        if cc.is_empty() {
            ev_curve_deleted.send(CurveDeletedEvent {
                curve_index: *curve_index,
            });
        } else {
            g_cc.push((*curve_index, cc));
        }
    }

    for (curve_index, cc) in g_cc {
        // Update curves
        for j in 0..cc.len() {
            if j == 0 {
                wall_manager.get_mut(curve_index).unwrap().curve = cc[0].clone();
                ev_curve_changed.send(CurveChangedEvent {
                    curve_index: curve_index,
                });
            } else {
                let index = wall_manager.new_wall(cc[j].clone());
                ev_curve_changed.send(CurveChangedEvent { curve_index: index });
            }
        }
    }
}

fn vec2_x0y(v: Vec2) -> Vec3 {
    Vec3::new(v.x, 0.0, v.y)
}

fn vec3_xz(v: Vec3) -> Vec2 {
    Vec2::new(v.x, v.z)
}

// TODO: replace with an analytical solution (same for `arch_layout_bricks`)
fn circle_segment_intersection(
    seg_start: Vec2,
    seg_end: Vec2,
    circle_center: Vec2,
    circle_radius: f32,
) -> Vec2 {
    let subdivs = 50;

    let mut min_d = (circle_radius - seg_start.distance(circle_center)).abs();

    for i in 1..(subdivs + 1) {
        let t = (i as f32) / (subdivs as f32);
        let p = seg_start.lerp(seg_end, t);

        let d = (circle_radius - p.distance(circle_center)).abs();
        //dbg!(t);
        //dbg!(circle_radius);
        //dbg!(p.distance(circle_center));

        if d < min_d {
            min_d = d;
        } else {
            // if distance started growing, then its the closest we are to intersection
            return p;
        }
    }

    seg_end

    /*
    let d = seg_end - seg_start;
    let f = seg_start - circle_center;
    let r = circle_radius;

    let a = d.dot(d);
    let b = 2.0 * f.dot(d);
    let c = f.dot(f) - r * r;

    let discriminant = b * b - 4.0 * a * c;

    dbg!(discriminant);

    if discriminant < 0.0 {
        return None;
    } else {
        let t1 = (-b - discriminant) / (2.0 * a);
        let t2 = (-b + discriminant) / (2.0 * a);

        dbg!(t1);
        dbg!(t2);

        if t1 >= 0.0 && t1 <= 1.0 {
            return Some(t1 * f);
        }

        if t2 >= 0.0 && t2 <= 1.0 {
            return Some(t2 * f);
        }

        return None;
    }
    */
}
