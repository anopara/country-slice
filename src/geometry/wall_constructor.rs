use crate::{components::transform::Transform};
use fastrand::Rng;
use glam::{Mat3, Quat, Vec2, Vec3};

use super::curve::Curve;

// could be interesting for the bricks offset https://www.iquilezles.org/www/articles/voronoise/voronoise.htm

const BRICK_WIDTH: f32 = 0.2;
const BRICK_WIDTH_VARIANCE: f32 = 0.14;

const BRICK_HEIGHT: f32 = 0.2;
const BRICK_HEIGHT_VARIANCE: f32 = 0.09;

const BRICK_DEPTH: f32 = 0.2;
const BRICK_DEPTH_VARIANCE: f32 = 0.05;

const WALL_HEIGHT: f32 = 1.4;

pub struct WallConstructor;

impl WallConstructor {
    pub fn from_curve(curve: &Curve) -> Vec<Brick> {
        let rng = fastrand::Rng::with_seed(0);

        let wall_length: f32 = curve.length;
        
        let row_count = (WALL_HEIGHT / BRICK_HEIGHT).floor() as usize;
        let rows  = random_splits(row_count, BRICK_HEIGHT_VARIANCE / WALL_HEIGHT, &rng);
        let bricks_per_row = (wall_length / BRICK_WIDTH).ceil() as usize; // this needs ceil, so that we always draw a brickwall if a curve is given, even if the bricks are too short

        let mut bricks = Vec::new();
        for (i, row_u) in rows.iter().enumerate() {

            let brick_height = if let Some(next_row_u) = rows.get(i+1) {
                (next_row_u - row_u) * WALL_HEIGHT
            } else {
                BRICK_HEIGHT + (rng.f32()-0.5) * BRICK_HEIGHT_VARIANCE
            };

            let brick_widths = random_splits(bricks_per_row, BRICK_WIDTH_VARIANCE / wall_length, &rng);

             // Bricks in curve space
            let mut brick_row: Vec<Brick> = Vec::new();
            for (j, this_u) in brick_widths.iter().enumerate() {
                if let Some(next_u) = brick_widths.get(j+1) {
                    // if its the last row, randomly skip some bricks!
                    if i == rows.len()-1 {
                        if rng.f32() < 0.35 {
                            continue;
                        }
                    }

                    let brick_depth = BRICK_DEPTH + (rng.f32()-0.5) * BRICK_DEPTH_VARIANCE;
                    //random chance to split horizontally into two bricks (except top row)
                    if rng.f32() < 0.4 && i != rows.len()-1  {
                        let range = (0.3, 0.7);
                        let random_split = rng.f32() * (range.1 - range.0) + range.0;
                        let pivot_u = (next_u + this_u) / 2.0;
                        let height_u_1 = brick_height / WALL_HEIGHT * random_split;
                        let height_u_2 = brick_height / WALL_HEIGHT * (1.0-random_split);
                        let pivot_v_1 = row_u + height_u_1 / 2.0;
                        let pivot_v_2 = (row_u + brick_height / WALL_HEIGHT) - height_u_2 / 2.0;
                        let width_u = next_u - this_u;
                        let width_ws = width_u * wall_length;
                        for (height, pivot_v, _idx) in vec![(height_u_1, pivot_v_1, i*2), (height_u_2, pivot_v_2, i*2+1)] {
                            brick_row.push(Brick {
                                //row_count: row_count * 2, // HACK: multiple it by two, to account for random splits that create in-between rows
                                //row_id_bottom: idx,
                                //row_id_top: idx+1,
                                pivot_uv: Vec2::new(pivot_u, pivot_v),
                                bounds_uv: Vec2::new(width_u, height),
                                transform: Transform {
                                    translation:  Vec3::new(pivot_u*wall_length, 0.0, 0.0),
                                    rotation: Quat::IDENTITY,
                                    scale: Vec3::new(width_ws, height * WALL_HEIGHT, brick_depth)
                                }
                            });
                        }
                    } else {
                        let pivot_u = (next_u + this_u) / 2.0;
                        let width_u = next_u - this_u;
                        let width_ws = width_u * wall_length;
                        brick_row.push(Brick {
                            pivot_uv: Vec2::new(pivot_u, row_u + brick_height / WALL_HEIGHT / 2.0),
                            bounds_uv: Vec2::new(width_u, brick_height / WALL_HEIGHT), 
                            transform: Transform { scale: Vec3::new(width_ws, brick_height, brick_depth),
                            translation: Vec3::new(pivot_u*wall_length, 0.0, 0.0),
                            rotation: Quat::IDENTITY}
                        });
                    }
                }
            }

            // Transform bricks into world space
            for brick in &mut brick_row {
                brick.transform.translation = curve.get_pos_at_u(brick.pivot_uv.x);
                brick.transform.translation.y = brick.pivot_uv.y * WALL_HEIGHT; //row_u * WALL_HEIGHT + brick_height / 2.0;

                let curve_tangent = curve.get_tangent_at_u(brick.pivot_uv.x);
                let normal = curve_tangent.cross(Vec3::Y);
                brick.transform.rotation = Quat::from_mat3(&Mat3::from_cols(curve_tangent, Vec3::Y, normal));
            }

            bricks.extend(brick_row);
        }

        bricks
    }
}

pub struct Brick {
    pub bounds_uv: Vec2,
    pub pivot_uv: Vec2,
    pub transform: Transform, 
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
