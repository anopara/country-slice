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

pub struct UiPrompt {
    pub padding: usize, // padding of the interaction bounding box in screen-space in pixels
    pub debug_preview: bevy_ecs::prelude::Entity,
}

pub struct UiPromptDebugPreview; //2D line strip to draw to show the bounding box in screenspace
