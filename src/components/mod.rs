pub mod drawable;
pub mod transform;

pub use drawable::*;
pub use transform::*;

// Mark the cube that is the preview of mouse raycast intersection
pub struct MousePreviewCube;

pub struct CursorRaycast(pub glam::Vec3);

pub struct DisplayTestMask;

// component
pub struct IndirectDraw;

//
pub struct RoadComponent;
