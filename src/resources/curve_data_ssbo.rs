use glam::Vec3;

// For storing curve data to pass to GPU
pub struct CurveSSBOCache(pub Vec<CurveDataSSBO>);

impl CurveSSBOCache {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CurveDataSSBO {
    pub points_count: u32,
    pub pad0: u32,
    pub pad1: u32,
    pub pad2: u32,
    pub positions: [[f32; 4]; 1000], //buffer
}

impl CurveDataSSBO {
    pub fn from(curve: &crate::geometry::curve::Curve) -> Self {
        let points_count = curve.points.len() as u32;
        let mut positions = [[0.0; 4]; 1000];

        positions.iter_mut().enumerate().for_each(|(i, p)| {
            *p = curve
                .points
                .get(i)
                .unwrap_or(&Vec3::ZERO)
                .extend(1.0)
                .to_array()
        });

        Self {
            points_count,
            pad0: 0,
            pad1: 0,
            pad2: 0,
            positions,
        }
    }
}
