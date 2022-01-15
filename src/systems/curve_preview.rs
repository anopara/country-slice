use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{DrawableMeshBundle, GLDrawMode, Transform},
    geometry::curve::Curve,
    render::mesh::Mesh,
    resources::{events::CurveDeletedEvent, wall_manager::WallManager, CurveChangedEvent},
};

const CURVE_SHOW_DEBUG: bool = true;

pub fn curve_preview(
    mut ev_curve_changed: EventReader<CurveChangedEvent>,
    mut ev_curve_deleted: EventReader<CurveDeletedEvent>,
    mut wall_manager: ResMut<WallManager>,

    query: Query<&Handle<Mesh>>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,

    mut commands: Commands,
) {
    if !CURVE_SHOW_DEBUG {
        return;
    }

    for ev in ev_curve_changed.iter() {
        // check if there is an entity associated with the curve
        let (curve, maybe_ent) = &wall_manager.curves[ev.curve_index];
        if let Some(ent) = maybe_ent {
            // if yes, update the mesh
            let mesh_handle = query.get(*ent).unwrap();
            update_curve_debug_mesh(curve, mesh_handle, &mut assets_mesh);
        } else {
            // if not, make a new entity
            wall_manager.curves[ev.curve_index].1 = Some(new_curve_entity(
                &mut assets_mesh,
                &assets_shader,
                &mut commands,
            ));
        }
    }

    // DELETE SYSTEM ---------------------------------------------------------------
    // This shifts all indices! we need to introduce a hash for curves
    let mut indices_to_remove = Vec::new();
    for ev in ev_curve_deleted.iter() {
        // Clear out preview entity if there is one
        if let Some(preview_ent) = &wall_manager.curves[ev.curve_index].1 {
            commands.entity(*preview_ent).despawn();
        }
        // Remove the curve entry
        indices_to_remove.push(ev.curve_index);
    }

    if !indices_to_remove.is_empty() {
        indices_to_remove.sort();
        wall_manager.curves =
            remove_sorted_indices(std::mem::take(&mut wall_manager.curves), indices_to_remove);
    }

    // --------------------------------------------------------------------------
}

fn remove_sorted_indices<T>(
    v: impl IntoIterator<Item = T>,
    indices: impl IntoIterator<Item = usize>,
) -> Vec<T> {
    let v = v.into_iter();
    let mut indices = indices.into_iter();
    let mut i = match indices.next() {
        None => return v.collect(),
        Some(i) => i,
    };
    let (min, max) = v.size_hint();
    let mut result = Vec::with_capacity(max.unwrap_or(min));

    for (j, x) in v.into_iter().enumerate() {
        if j == i {
            if let Some(idx) = indices.next() {
                i = idx;
            }
        } else {
            result.push(x);
        }
    }

    result
}

fn new_curve_entity(
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
    assets_shader: &Res<AssetShaderLibrary>,
    commands: &mut Commands,
) -> Entity {
    let curve_mesh_handle = assets_mesh.add(Mesh::new().into());
    let shader = assets_shader
        .get_handle_by_name("vertex_color_shader")
        .unwrap();

    commands
        .spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: curve_mesh_handle,
            shader,
            transform: Transform::identity(),
        })
        .insert(GLDrawMode(gl::LINE_STRIP))
        .id()
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
