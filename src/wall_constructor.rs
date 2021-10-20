use crate::curve::Curve;
use bevy::prelude::*;

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

const WALL_HEIGHT: f32 = 2.0;

pub struct WallConstructor;

impl WallConstructor {
    pub fn from_curve(curve: &Curve) {
        let wall_length: f32 = curve.length();

        let wall_sizes = irregular_row(wall_length, BRICK_WIDTH, BRICK_WIDTH_VARIANCE, 0);

        // TODO:
        // 1. from wall sizes find pivots for the bricks
        // 2. from pivots, convert them into curve_u position and sample WS position from curve
        // 3. from pivots, convert them into curve_u position and construct Quat based on tangent
        // 4. return transforms from which meshes can be placed
    }
}

fn irregular_row(target_length: f32, piece_mean: f32, piece_variance: f32, seed: u64) -> Vec<f32> {
    let mut total_length = 0.0;
    let mut pieces = Vec::new();

    let rng = fastrand::Rng::with_seed(seed);

    // keep generating until we have something larger than the target length
    while total_length < target_length {
        let new_piece = piece_mean + (rng.f32() - 0.5) * piece_variance;
        pieces.push(new_piece);
        total_length += new_piece;
    }

    // re-normalize the generated pieces so that they fit perfectly into the target length
    pieces
        .iter()
        .map(|p| p * target_length / total_length)
        .collect()
}
