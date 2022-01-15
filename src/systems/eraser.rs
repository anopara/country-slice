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

            let p1_is_inside = cursor_ws.distance(p1) < ERASE_BRUSH_SIZE;
            let p2_is_inside = cursor_ws.distance(p2) < ERASE_BRUSH_SIZE;

            match (p1_is_inside, p2_is_inside) {
                (false, false) => new_curves[new_curve_last_index].push(p1),
                (false, true) => {
                    // TODO: find exact intersection
                    new_curves[new_curve_last_index].push(p1);
                    // this is the end of the curve outside the brush stroke
                    new_curves.push(Vec::new());
                    new_curve_last_index += 1;
                }
                (true, false) => {
                    // this is the beginning of the curve outside the brush stroke
                    // TODO: find exact intersection
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
        // TODO: clear memory of lefotver VAOs
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
                log::info!("Event: Curve {} has changed", curve_index);
            } else {
                let index = wall_manager.new_wall(cc[j].clone());
                ev_curve_changed.send(CurveChangedEvent { curve_index: index });
                log::info!("Event: Curve {} has changed", index);
            }
        }
    }

    /*
    // Go through all the curves
    //let mut curves_to_replace = Vec::new();
    let mut stuff = Vec::new();
    {
        puffin::profile_scope!("find deletions");
        for (i, (curve, ent)) in wall_manager.curves.iter().enumerate() {
            let pts_to_delete: Vec<usize> = curve
                .points
                .iter()
                .enumerate()
                .filter_map(|(j, pt)| {
                    if cursor_ws.distance(*pt) < ERASE_BRUSH_SIZE {
                        Some(j)
                    } else {
                        // if so delete this point
                        None
                    }
                })
                .collect();

            if !pts_to_delete.is_empty() {
                stuff.push((i, pts_to_delete, ent.clone()))
            }
        }
    }

    {
        puffin::profile_scope!("split");

        for (curve_indx, pts_to_delete, ent) in stuff.iter() {
            //dbg!(curve_indx);
            //dbg!(pts_to_delete);
            //panic!();

            let curve = &wall_manager.curves[*curve_indx].0;

            let mut splits = vec![(0, pts_to_delete[0])];
            for (i, p_index) in pts_to_delete.iter().enumerate() {
                if let Some(next) = pts_to_delete.get(i + 1) {
                    if p_index + 1 != *next {
                        // that's a break!
                        splits.push((p_index + 1, *next));
                    }
                } else {
                    splits.push((p_index + 1, curve.points.len()));
                }
            }

            //(TODO: actually delete the lingering points!) otherwise we have these leftovers of 2 segments everywhere
            // those are probably lingering SSBOs and VAOs that are never properly cleared out
            // need to have a way to call a full curve deletion

            // check if no degenerate curves
            splits = splits
                .iter()
                .filter_map(move |(start, end)| {
                    if end - start > 2 {
                        Some((*start, *end))
                    } else {
                        None
                    }
                })
                .collect();

            //dbg!(splits);
            let new_curves: Vec<Vec<Vec3>> = splits
                .iter()
                .map(|(start, end)| curve.points[*start..*end].to_vec())
                .collect();

            // Update curves
            for j in 0..new_curves.len() {
                if j == 0 {
                    wall_manager.curves[*curve_indx] = (Curve::from(new_curves[0].clone()), *ent);
                    ev_curve_changed.send(CurveChangedEvent {
                        curve_index: *curve_indx,
                    });
                } else {
                    wall_manager
                        .curves
                        .push((Curve::from(new_curves[j].clone()), None));
                    ev_curve_changed.send(CurveChangedEvent {
                        curve_index: wall_manager.curves.len() - 1,
                    });
                }
            }
        }
    }
    */
}
