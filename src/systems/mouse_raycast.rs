use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use glam::{Mat4, Vec2, Vec3};

use crate::components::transform::Transform;
use crate::components::MousePreviewCube;
use crate::window_events::{CursorMoved, WindowSize};
use crate::CursorRaycast;

use crate::render::camera::{Camera, MainCamera};

pub fn mouse_raycast(
    mut cube_query: Query<(&mut MousePreviewCube, &mut Transform)>,
    mut cursor: EventReader<CursorMoved>,
    mut cursor_ws_cache: ResMut<CursorRaycast>,
    main_camera: Res<MainCamera>,
    window_size: Res<WindowSize>,
) {
    if let Some(cursor_latest) = cursor.iter().last() {
        let (cursor_ws, ray) = from_screenspace_to_ws(
            cursor_latest.pos,
            Vec2::new(window_size.width as f32, window_size.height as f32),
            &main_camera.camera,
        );

        // ray-plane intersction, solving for (P + d * ray).y = 0.0
        let p = cursor_ws + ray * (-cursor_ws.y / ray.y);
        *cursor_ws_cache = CursorRaycast(p);

        // Update preview cube
        for (_, mut transform) in cube_query.iter_mut() {
            transform.translation = p;
        }
    }
}

// from bevy_mod_raycast
// https://github.com/aevyrie/bevy_mod_raycast/blob/master/src/lib.rs
pub fn from_screenspace_to_ws(
    cursor_pos_screen: Vec2,
    screen_size: Vec2,
    camera: &Camera,
) -> (Vec3, Vec3) {
    let camera_position = camera.transform;

    let projection_matrix = camera.perspective_projection;

    // Normalized device coordinate cursor position from (-1, -1, -1) to (1, 1, 1)
    let cursor_ndc = (Vec2::new(cursor_pos_screen.x, 1.0 - cursor_pos_screen.y) / screen_size)
        * 2.0
        - Vec2::from([1.0, -1.0]);
    let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
    let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

    // Use near and far ndc points to generate a ray in world space
    // This method is more robust than using the location of the camera as the start of
    // the ray, because ortho cameras have a focal point at infinity!
    let ndc_to_world: Mat4 = camera_position * projection_matrix.inverse();
    let cursor_pos_ws = ndc_to_world.project_point3(cursor_ndc.extend(0.0));
    let cursor_pos_near: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_near);
    let cursor_pos_far: Vec3 = ndc_to_world.project_point3(cursor_pos_ndc_far);
    let ray_direction = cursor_pos_far - cursor_pos_near;
    //Some(Ray3d::new(cursor_pos_near, ray_direction))
    //let (s, r, _) = camera.transform.to_scale_rotation_translation();

    //GlobalTransform {
    //    translation: bevy_math::f32::Vec3::new(cursor_pos_ws.x, cursor_pos_ws.y, cursor_pos_ws.z),
    //    rotation: bevy_math::f32::Quat::from_xyzw(r.x, r.y, r.z, r.w),
    //    scale: bevy_math::f32::Vec3::new(s.x, s.y, s.z),
    //}

    (cursor_pos_ws, ray_direction.normalize())
}

/* Note: if I ever want to use bevy_mod_raycast, I'd probably have to fork it
for raycast in query.iter() {
    //println!("raycast");
    if let Some((_, intersection)) = raycast.intersect_top() {
        println!("intersection {:?}", intersection);
        for (_, mut transform) in cube_query.iter_mut() {
            transform.translation = intersection.position();
        }
    }
}
 */

/*
// Update our `RayCastSource` with the current cursor position every frame.
pub fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<(&mut RayCastSource<MyRaycastSet>, &mut GlobalTransform)>,
    main_camera: Res<MainCamera>,
) {
    for (mut pick_source, mut transform) in &mut query.iter_mut() {
        // Grab the most recent cursor event if it exists:
        if let Some(cursor_latest) = cursor.iter().last() {
            pick_source.cast_method = RayCastMethod::Transform;

            // Update its Global Transform
            *transform = from_screenspace(
                cursor_latest.pos,
                Vec2::new(1600.0, 1200.0),
                &main_camera.camera,
            )
        }
    }
}
*/
