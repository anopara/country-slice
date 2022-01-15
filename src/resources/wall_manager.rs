use std::collections::HashMap;

use bevy_ecs::prelude::{Commands, Entity};

use crate::geometry::curve::Curve;

pub const RESAMPLING: f32 = 0.2;
pub const SMOOTHING_STEPS: usize = 50;

pub struct Wall {
    pub curve: Curve,
    pub curve_preview_entity: Option<Entity>,
    pub wall_entity: Option<Entity>,
    pub shadow_entity: Option<Entity>,
}

impl Wall {
    pub fn from(v: Curve) -> Self {
        Self {
            curve: v,
            curve_preview_entity: None,
            wall_entity: None,
            shadow_entity: None,
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

    pub fn get(&self, index: usize) -> Option<&Wall> {
        self.walls.get(&index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Wall> {
        self.walls.get_mut(&index)
    }

    pub fn remove_entry(&mut self, index: usize, commands: &mut Commands) {
        let wall_to_remove = self.get(index).expect(&format!(
            "Remove entry failed: index {} doesnt exist",
            index
        ));

        despawn_if_exists(wall_to_remove.curve_preview_entity, commands);
        despawn_if_exists(wall_to_remove.wall_entity, commands);
        despawn_if_exists(wall_to_remove.shadow_entity, commands);
    }
}

fn despawn_if_exists(ent: Option<Entity>, commands: &mut Commands) {
    if let Some(ent) = ent {
        commands.entity(ent).despawn();
    }
}
