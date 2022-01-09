use bevy_ecs::prelude::*;
use glam::Vec3;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{
        drawable::{DrawableMeshBundle, GLDrawMode},
        transform::Transform,
        EditingHandle, TriggerArea, TriggerAreaPreview,
    },
    geometry::curve::Curve,
    render::mesh::Mesh,
    resources::WallManager,
    systems::mode_manager::ActiveCurve,
    CursorRaycast,
};

use super::mode_manager::Mode;

const CURVE_SHOW_DEBUG: bool = true;

pub fn draw_curve(
    mode: Res<Mode>,

    mut query: Query<&Handle<Mesh>>,
    mut query_2: Query<&mut TriggerArea>, // for editing handles
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,

    mut commands: Commands,

    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
) {
    puffin::profile_function!();

    /*
    // store for editing handles, they need to know about Y (TODO: they are not going to be updated if terrain changes!)
    let cursor_ws_w_y = cursor_ws.0;
    // Remove y component from the cursor-terrain raycast position
    let mut cursor_ws = cursor_ws.0;
    //cursor_ws.y = 0.0;
    */

    match &*mode {
        Mode::None => {}
        Mode::StartNewCurve => {
            wall_manager
                .curves
                .push(start_curve(&mut assets_mesh, &assets_shader, &mut commands));

            //TODO: add a second handle for the end, and it should update its transform as we draw the curve

            let mut trigger_area_comp =
                TriggerArea::new(20, Transform::from_translation(cursor_ws.0));
            trigger_area_comp.add_screen_space_preview(
                &mut assets_mesh,
                &assets_shader,
                &mut commands,
            );
            //trigger_area_comp.add_world_space_preview(
            //    &mut assets_mesh,
            //    &assets_shader,
            //    &mut commands,
            //);

            let entity = commands
                .spawn()
                .insert(EditingHandle::new(wall_manager.curves.len() - 1))
                .insert(trigger_area_comp)
                .id();

            wall_manager.editing_handles.push(entity);
        }
        Mode::DrawingCurve(active_curve) => {
            let active_curve_index = match active_curve {
                ActiveCurve::Last => wall_manager.curves.len() - 1,
                ActiveCurve::Index(index) => *index,
            };

            let (active_curve, preview_entity) =
                wall_manager.curves.get_mut(active_curve_index).unwrap();

            let intersection = cursor_ws.0;

            const DIST_THRESHOLD: f32 = 0.001;

            if active_curve
                .points
                .last()
                // if curve  had points, only add if the distance is larger than X
                .map(|last| intersection.distance(*last) > DIST_THRESHOLD)
                // if curve  has no points, add this point
                .unwrap_or(true)
            {
                active_curve.add(intersection);

                // Update the curve debug preview mesh, if its present
                if let Some(Ok(mesh_handle)) = preview_entity.map(|ent| query.get_mut(ent)) {
                    update_curve_debug_mesh(&active_curve, mesh_handle, &mut assets_mesh);
                }

                // Update editing handles
                let handle_entity = wall_manager
                    .editing_handles
                    .get_mut(active_curve_index)
                    .unwrap();

                // TODO: separate Trigger Area & Transform components
                let mut handle_trigger = query_2.get_mut(*handle_entity).unwrap();
                handle_trigger.update_transform(intersection);
            }
        }

        Mode::EditingCurve(_) => todo!(),
    }
}

fn update_curve_debug_mesh(
    curve: &Curve,
    mesh_handle: &Handle<Mesh>,
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
) {
    let mesh = assets_mesh.get_mut(*mesh_handle).expect("MEOW####");
    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        curve
            .points
            .iter()
            .map(|p| [p.x, p.y + 0.01, p.z])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.set_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[1.0, 0.0, 0.0]; curve.points.len()],
    );
    mesh.set_indices((0..curve.points.len()).map(|i| i as u32).collect());
}

fn start_curve(
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
    assets_shader: &Res<AssetShaderLibrary>,
    commands: &mut Commands,
) -> (Curve, Option<Entity>) {
    let curve = Curve::new();

    let preview_entity = if CURVE_SHOW_DEBUG {
        let curve_mesh_handle = assets_mesh.add(Mesh::new().into());
        let shader = assets_shader
            .get_handle_by_name("vertex_color_shader")
            .unwrap();

        Some(
            commands
                .spawn()
                .insert_bundle(DrawableMeshBundle {
                    mesh: curve_mesh_handle,
                    shader,
                    transform: Transform::identity(),
                })
                .insert(GLDrawMode(gl::LINE_STRIP))
                .id(),
        )
    } else {
        None
    };

    (curve, preview_entity)
}
