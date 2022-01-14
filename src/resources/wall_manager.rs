use bevy_ecs::prelude::Entity;

use crate::geometry::curve::Curve;

pub struct WallManager {
    pub temp_curve: Option<Curve>, // raw curve data being currently drawn by the user without any smoothing or resmapling
    pub curves: Vec<(Curve, Option<Entity>)>,
    pub walls: Vec<Entity>,
    pub shadows: Vec<Entity>,
}

impl WallManager {
    pub fn new() -> Self {
        Self {
            temp_curve: None,
            curves: Vec::new(),
            walls: Vec::new(),
            shadows: Vec::new(),
        }
    }
}
