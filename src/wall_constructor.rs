use crate::curve::Curve;
use bevy::prelude::*;
use fastrand::Rng;

// 1. measure the smoothed curve
// 2. construct a grid based on measurements
// 3. randomly determine the width of the grid elements (split equally, and then randomly perturb the U value)
// 4. construct transforms
// 5. project the transforms back on the curve
// 6. place meshes

// NEXT STEP: split the trasnforms into two randomly

const BRICK_WIDTH: f32 = 0.2;
const BRICK_WIDTH_VARIANCE: f32 = 0.14;

const BRICK_HEIGHT: f32 = 0.2;
const BRICK_HEIGHT_VARIANCE: f32 = 0.06;

const BRICK_DEPTH: f32 = 0.2;
const BRICK_DEPTH_VARIANCE: f32 = 0.03;

const WALL_HEIGHT: f32 = 2.0;

pub struct WallConstructor;

impl WallConstructor {
    pub fn from_curve(curve: &Curve) -> Vec<Brick> {
        let rng = fastrand::Rng::with_seed(0);

        let wall_length: f32 = curve.length;
        
        let row_count = (WALL_HEIGHT / BRICK_HEIGHT).floor() as usize;
        let rows  = random_splits(row_count, BRICK_HEIGHT_VARIANCE / WALL_HEIGHT, &rng);
        let bricks_per_row = (wall_length / BRICK_WIDTH).floor() as usize;
        
        let mut bricks = Vec::new();
        for row_u in rows.iter(){

            let brick_widths = random_splits(bricks_per_row, BRICK_WIDTH_VARIANCE / wall_length, &rng);
             // Bricks in curve space
            let mut brick_row: Vec<Brick> = brick_widths.iter().enumerate().filter_map(|(i, this_u)| if let Some(next_u) = brick_widths.get(i+1) {
                let pivot_u = (next_u + this_u) / 2.0;
                let width_u = next_u - this_u;
                let width_ws = width_u * wall_length;
                Some(Brick {
                    pivot_u,
                    scale: Vec3::new(width_ws, BRICK_HEIGHT, BRICK_DEPTH + (rng.f32()-0.5) * BRICK_DEPTH_VARIANCE),
                    position: Vec3::new(pivot_u*wall_length, 0.0, 0.0),
                    rotation: Quat::IDENTITY
                })
            } else {
                None
            }).collect();

            // Transform bricks into world space
            for brick in &mut brick_row {
                brick.position = curve.get_pos_at_u(brick.pivot_u);
                brick.position.y = row_u * WALL_HEIGHT + BRICK_HEIGHT / 2.0;

                let curve_tangent = curve.get_tangent_at_u(brick.pivot_u);
                let normal = curve_tangent.cross(Vec3::Y);
                brick.rotation = Quat::from_rotation_mat3(&Mat3::from_cols(curve_tangent, Vec3::Y, normal));
            }

            bricks.extend(brick_row);
        }

        bricks
    }
}

pub struct Brick {
    pub pivot_u: f32,
    pub scale: Vec3,
    pub position: Vec3,
    pub rotation: Quat,
}


// random splits in [0;1] range
fn random_splits(splits: usize, variance_u: f32, rng: &Rng) -> Vec<f32> {
     // uniform points in curve_u
     let row_u: Vec<f32> = (0..(splits+1)).map(|i| (i as f32) / (splits as f32)).collect();

     // perturb
     row_u.iter().enumerate().map(|(i, u)| 
         // skip first and last points
         if i != 0 && i != row_u.len()-1 { 
             u + (rng.f32() - 0.5) * variance_u
         } else { 
             *u
     }).collect()

}




/*
fn brick_row(target_length: f32, piece_mean: f32, piece_variance: f32, seed: u64) -> Vec<Brick> {
    let mut total_width = 0.0;
    let mut brick_widths = Vec::new();

    let rng = fastrand::Rng::with_seed(seed);

    // keep generating until we have something larger than the target length
    while total_width < target_length {
        let width = piece_mean + (rng.f32() - 0.5) * piece_variance;
        total_width += width;

        brick_widths.push(width);
    }

    // re-normalize the generated pieces so that they fit perfectly into the target length
    brick_widths = brick_widths
        .iter()
        .map(|p| p * target_length / total_width)
        .collect();

    //
    brick_widths.iter().map(|w| )
}
*/
