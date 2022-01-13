pub mod compute_arches_indirect;
pub mod compute_paths_mask;
pub mod curve_segments_pass;
pub mod events;
pub mod terrain;
pub mod wall_manager;

//use bevy_app::AppBuilder;
pub use compute_arches_indirect::*;
pub use compute_paths_mask::*;
pub use curve_segments_pass::*;
pub use events::*;
pub use terrain::*;
pub use wall_manager::*;

//pub fn add_events(app: &mut AppBuilder) -> &mut AppBuilder {
//    app.add_event::<CurveChangedEvent>()
//}
