use std::collections::HashMap;

use bevy_ecs::prelude::Entity;

use crate::{
    asset_libraries::{Handle, HandleId},
    geometry::curve::Curve,
};

pub const RESAMPLING: f32 = 0.2;
pub const SMOOTHING_STEPS: usize = 50;

pub struct Wall {
    pub curve: Curve,
    pub curve_preview_entity: Option<Entity>,
    pub wall_entity: Option<Entity>,
    pub shadow: Option<Entity>,
}

impl Wall {
    pub fn from(v: Curve) -> Self {
        Self {
            curve: v,
            curve_preview_entity: None,
            wall_entity: None,
            shadow: None,
        }
    }
}

pub struct WallManager {
    pub temp_curve: Option<Curve>,
    pub walls: HashMap<usize, Wall>,

    pub max_index: usize,
}

impl WallManager {
    pub fn new() -> Self {
        Self {
            temp_curve: None,
            walls: HashMap::new(),
            max_index: 0,
        }
    }

    pub fn new_wall(&mut self, curve: Curve) -> usize {
        self.max_index += 1;

        self.temp_curve = Some(Curve::new());
        self.walls.insert(self.max_index, Wall::from(curve));

        self.max_index
    }

    //pub fn last(&self) -> Option<&Wall> {
    //    self.walls.get(&self.max_index)
    //}

    pub fn last_mut(&mut self) -> Option<&mut Wall> {
        self.walls.get_mut(&self.max_index)
    }
}
