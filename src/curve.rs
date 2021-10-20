use bevy::prelude::*;

pub struct Curve {
    pub points: Vec<Vec3>,
}

impl Curve {
    pub fn length(&self) -> f32 {
        self.points
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                self.points
                    .get(idx + 1)
                    .map(|next_p| (*next_p - *p).length())
                    .unwrap_or(0.0)
            })
            .sum()
    }

    // Curve segment is defined by start_point_index and end_point_index
    fn get_curve_segment_from_u(&self, u: f32) -> (usize, usize) {
        let remaped_u = u * (self.points.len() as f32 - 1.0);
        let closest_index_1 = remaped_u.floor() as usize;
        let closest_index_2 = remaped_u.ceil() as usize;

        // If the u value is _exactly_ as the point location
        if closest_index_1 == closest_index_2 {
            // if it's the last point, return the previous point + last point
            if u == 1.0 {
                (closest_index_1 - 1, closest_index_1)
            }
            // otherwise, return this exact point and the next
            else {
                (closest_index_1, closest_index_1 + 1)
            }
        } else {
            (closest_index_1, closest_index_2)
        }
    }

    pub fn get_pos_at_u(&self, u: f32) -> Vec3 {
        assert!(u <= 1.0 && u >= 0.0, "u is in incorrect range");

        let (idx1, idx2) = self.get_curve_segment_from_u(u);

        let dir = self.points[idx2] - self.points[idx1];
        let remaped_u = u * (self.points.len() as f32 - 1.0);
        let mag = remaped_u - (idx1 as f32);

        self.points[idx1] + dir * mag
    }

    pub fn get_tangent_at_u(&self, u: f32) -> Vec3 {
        assert!(u <= 1.0 && u >= 0.0, "u is in incorrect range");

        let (idx1, idx2) = self.get_curve_segment_from_u(u);

        (self.points[idx2] - self.points[idx1]).normalize()
    }
}
