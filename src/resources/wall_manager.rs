use bevy_ecs::prelude::Entity;

use crate::geometry::curve::Curve;

pub struct WallManager {
    pub curves: Vec<(Curve, Option<Entity>)>,
    pub walls: Vec<Entity>,
    pub shadows: Vec<Entity>,
    pub editing_handles: Vec<(Entity, Entity)>, // start and end of the curve
}

impl WallManager {
    pub fn new() -> Self {
        Self {
            curves: Vec::new(),
            walls: Vec::new(),
            shadows: Vec::new(),
            editing_handles: Vec::new(),
        }
    }
}
