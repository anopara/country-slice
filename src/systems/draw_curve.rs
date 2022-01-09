use bevy_ecs::prelude::*;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{
        drawable::{DrawableMeshBundle, GLDrawMode},
        transform::Transform,
        EditingHandle, HandleLocation, TriggerArea,
    },
    geometry::curve::Curve,
    render::mesh::Mesh,
    resources::WallManager,
    systems::mode_manager::{ActiveCurve, AddPointsTo},
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

            let mut trigger_area_comp =
                TriggerArea::new(20, Transform::from_translation(cursor_ws.0));
            trigger_area_comp.add_screen_space_preview(
                &mut assets_mesh,
                &assets_shader,
                &mut commands,
            );

            let mut trigger_area_comp_2 =
                TriggerArea::new(20, Transform::from_translation(cursor_ws.0));
            trigger_area_comp_2.add_screen_space_preview(
                &mut assets_mesh,
                &assets_shader,
                &mut commands,
            );

            let start_handle = commands
                .spawn()
                .insert(EditingHandle::new(
                    wall_manager.curves.len() - 1,
                    HandleLocation::StartOfCurve,
                ))
                .insert(trigger_area_comp)
                .id();

            let end_handle = commands
                .spawn()
                .insert(EditingHandle::new(
                    wall_manager.curves.len() - 1,
                    HandleLocation::EndOfCurve,
                ))
                .insert(trigger_area_comp_2)
                .id();

            wall_manager
                .editing_handles
                .push((start_handle, end_handle));
        }
        Mode::DrawingCurve(active_curve, draw_mode) => {
            let active_curve_index = match active_curve {
                ActiveCurve::Last => wall_manager.curves.len() - 1,
                ActiveCurve::Index(index) => *index,
            };

            let (start_handle, end_handle) = wall_manager
                .editing_handles
                .get(active_curve_index)
                .unwrap()
                .clone();
            let (active_curve, preview_entity) =
                wall_manager.curves.get_mut(active_curve_index).unwrap();

            let active_curve_pt = match draw_mode {
                AddPointsTo::End => active_curve.points.len() - 1,
                AddPointsTo::Beginning => 0,
            };

            let intersection = cursor_ws.0;

            const DIST_THRESHOLD: f32 = 0.001;

            if active_curve
                .points
                .get(active_curve_pt)
                // if curve  had points, only add if the distance is larger than X
                .map(|last| intersection.distance(*last) > DIST_THRESHOLD)
                // if curve  has no points, add this point
                .unwrap_or(true)
            {
                match draw_mode {
                    AddPointsTo::End => active_curve.add(intersection),
                    AddPointsTo::Beginning => active_curve.add_to_front(intersection),
                }

                // Update the curve debug preview mesh, if its present
                if let Some(Ok(mesh_handle)) = preview_entity.map(|ent| query.get_mut(ent)) {
                    update_curve_debug_mesh(&active_curve, mesh_handle, &mut assets_mesh);
                }

                // Update editing handles
                query_2
                    .get_mut(start_handle)
                    .unwrap()
                    .update_transform(active_curve.points[0]);

                query_2
                    .get_mut(end_handle)
                    .unwrap()
                    .update_transform(*active_curve.points.last().unwrap());
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
