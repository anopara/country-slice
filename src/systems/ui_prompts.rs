use bevy_app::EventReader;
use bevy_ecs::prelude::*;
use glam::{Vec2, Vec3};

use crate::{
    components::{CursorRaycast, UiPrompt},
    render::camera::{Camera, MainCamera},
    window_events::{CursorMoved, WindowSize},
};

pub fn ui_prompts(
    cursor: EventReader<CursorMoved>,
    main_camera: Res<MainCamera>,
    window_size: Res<WindowSize>,
    query: Query<&UiPrompt>,
) {

    // TODO: for every vertex of the cube, transform it into screenspace
    // find the bounding box of them in screenspace
    // expand that box by X pixels padding
    // check if mouse is inside that 2d volume
    // print something to console if so
    // TODO: also start looking into events!
}

pub fn from_ws_to_screenspace(ws_pos: Vec3, screen_size: Vec2, camera: &Camera) -> Vec2 {
    let ws_to_clip = camera.perspective_projection;

    let clip_pos = ws_to_clip.transform_point3(ws_pos);

    // TODO: check if -1 to 1 range, aka in view

    let nss_pos = Vec2::new(clip_pos.x + 0.5, clip_pos.y + 0.5);

    nss_pos * screen_size
}
