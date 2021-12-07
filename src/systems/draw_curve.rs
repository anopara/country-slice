use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{
        drawable::{DrawableMeshBundle, GLDrawMode},
        transform::Transform,
    },
    geometry::curve::Curve,
    render::mesh::Mesh,
    ComputeDrawIndirectTest, CursorRaycast, WallManager,
};

const CURVE_SHOW_DEBUG: bool = true;

// for compute indirect
pub fn update_curve_ssbo(
    wall_manager: Res<WallManager>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut compute_indirect: ResMut<ComputeDrawIndirectTest>,
) {
    // HACK: ideally, this should be an eveent that tells that curves have been update and SSBO needs updating
    if mouse_button_input.pressed(MouseButton::Left) {
        // HACK: update all the curves, bc its easier (in the future, no need to redo the whole buffer....)
        let data: Vec<_> = wall_manager
            .curves
            .iter()
            .map(|(curve, _)| {
                if curve.points.len() > 0 {
                    let c = crate::CurveDataSSBO::from(&curve.clone().smooth(50).resample(0.2));
                    //println!("curve has {} points", c.points_count);
                    c
                } else {
                    // add empty
                    crate::CurveDataSSBO {
                        points_count: 0,
                        pad0: 0,
                        pad1: 0,
                        pad2: 0,
                        positions: [[0.0; 4]; 1000],
                    }
                }
            })
            .collect();

        compute_indirect.curves_buffer.update(&data);
    }
}

pub fn draw_curve(
    mut query: Query<&Handle<Mesh>>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,

    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,

    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
) {
    // If LMB was just pressed, start a new curve
    if mouse_button_input.just_pressed(MouseButton::Left) {
        wall_manager
            .curves
            .push(start_curve(&mut assets_mesh, &assets_shader, &mut commands));
    }
    // If LMB is pressed, continue the active curve
    else if mouse_button_input.pressed(MouseButton::Left) {
        let (active_curve, preview_entity) = wall_manager.curves.last_mut().unwrap();

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
        }
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
