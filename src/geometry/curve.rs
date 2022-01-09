use glam::Vec3;

#[derive(Clone)]
pub struct Curve {
    pub points: Vec<Vec3>,
    // cache u values upon creation
    pub points_u: Vec<f32>,
    pub length: f32,
}

impl Curve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            points_u: Vec::new(),
            length: 0.0,
        }
    }

    pub fn add_to_front(&mut self, pt: Vec3) {
        self.points.insert(0, pt);

        // TODO: this is slow & lazy
        *self = Self::from(self.points.clone());
    }

    pub fn add(&mut self, pt: Vec3) {
        self.points.push(pt);

        // TODO: this is slow & lazy
        *self = Self::from(self.points.clone());
    }

    pub fn from(points: Vec<Vec3>) -> Self {
        let length = points
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                points
                    .get(idx + 1)
                    .map(|next_p| (*next_p - *p).length())
                    .unwrap_or(0.0)
            })
            .sum();

        let mut length_traveled = 0.0;
        let mut points_u = Vec::new();

        for (idx, pt) in points.iter().enumerate() {
            points_u.push(length_traveled / length);
            if let Some(next_pt) = points.get(idx + 1) {
                length_traveled += (*next_pt - *pt).length();
            }
        }

        Self {
            points,
            points_u,
            length,
        }
    }

    pub fn smooth(mut self, smoothing_steps: usize) -> Self {
        if self.points.len() < 3 {
            return self;
        }

        for _ in 0..smoothing_steps {
            let mut current_iter_smooth = self.points.clone();
            for (i, current_pos) in self.points.iter().enumerate() {
                // skip first point
                if i == 0 {
                    continue;
                }

                if let (Some(prev_pos), Some(next_pos)) =
                    (self.points.get(i - 1), self.points.get(i + 1))
                {
                    let avg: Vec3 = (*prev_pos + *next_pos) / 2.0;
                    current_iter_smooth[i] = *current_pos + (avg - *current_pos) * 0.5;
                }
            }
            self.points = current_iter_smooth;
        }

        self
    }

    pub fn resample(self, segment_length: f32) -> Self {
        if segment_length >= self.length {
            return Curve::from(vec![self.points[0], *self.points.last().unwrap()]);
        }

        //  TODO: this is TEMPORARY & SLOW! this re-uses the `get_pos_at_u` which searches the curve from start every time
        let u_spacing = segment_length / self.length;
        let target_points = (1.0 / u_spacing).round() as usize;
        let target_u_spacing = 1.0 / (target_points as f32);

        Curve::from(
            (0..=target_points)
                .map(|i| self.get_pos_at_u((i as f32) * target_u_spacing))
                .collect(),
        )
    }

    // Curve segment is defined by start_point_index and end_point_index
    fn get_curve_segment_from_u(&self, u: f32) -> (usize, usize) {
        if u == 1.0 {
            return (self.points.len() - 2, self.points.len() - 1);
        } else if u == 0.0 {
            return (0, 1);
        }

        for (i, pt_u) in self.points_u.iter().enumerate() {
            if u <= *pt_u {
                return (i - 1, i);
            }
        }

        unreachable!()
    }

    pub fn get_pos_at_u(&self, u: f32) -> Vec3 {
        assert!(u <= 1.0 && u >= 0.0, "u is in incorrect range");

        let (idx1, idx2) = self.get_curve_segment_from_u(u);

        let dir = self.points[idx2] - self.points[idx1];
        let u_range = (self.points_u[idx1], self.points_u[idx2]);

        let mag = (u - u_range.0) / (u_range.1 - u_range.0);

        self.points[idx1] + dir * mag
    }

    pub fn get_tangent_at_u(&self, u: f32) -> Vec3 {
        assert!(u <= 1.0 && u >= 0.0, "u is in incorrect range");

        let (idx1, idx2) = self.get_curve_segment_from_u(u);

        (self.points[idx2] - self.points[idx1]).normalize()
    }
}
