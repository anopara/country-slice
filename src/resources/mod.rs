pub mod compute_arches_indirect;
pub mod compute_paths_mask;
pub mod curve_segments_pass;
pub mod terrain;
pub mod wall_manager;

pub use compute_arches_indirect::*;
pub use compute_paths_mask::*;
pub use curve_segments_pass::*;
pub use terrain::*;
pub use wall_manager::*;

pub struct LastHoveredTriggerArea(pub Option<bevy_ecs::prelude::Entity>);
