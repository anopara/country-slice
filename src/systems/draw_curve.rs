use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};
use glam::Vec3;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{
        drawable::{DrawableMeshBundle, GLDrawMode},
        transform::Transform,
        TriggerArea, TriggerAreaPreview,
    },
    geometry::curve::Curve,
    render::mesh::Mesh,
    resources::{LastHoveredTriggerArea, WallManager},
    CursorRaycast,
};

const CURVE_SHOW_DEBUG: bool = false;

pub fn draw_curve(
    last_hovered: Res<LastHoveredTriggerArea>, //editing handle

    mut query: Query<&Handle<Mesh>>,
    mut wall_manager: ResMut<WallManager>,
    cursor_ws: Res<CursorRaycast>,

    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,

    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
) {
    // store for editing handles, they need to know about Y (TODO: they are not going to be updated if terrain changes!)
    let cursor_ws_w_y = cursor_ws.0;
    // Remove y component from the cursor-terrain raycast position
    let mut cursor_ws = cursor_ws.0;
    cursor_ws.y = 0.0;

    puffin::profile_function!();

    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Check if we are continuing an old curve
        if let Some(trigger_entity) = last_hovered.0 {
            unimplemented!()
            // Check which curve we are continuing (do I need a hashmap? or trigger itself stores its parent entity?) triggers should only care for curves! wall construction is just a decorator on top of that core data
            // Are we continuing beginning or end?
        }
        // Otherwise, start a new curve
        else {
            wall_manager
                .curves
                .push(start_curve(&mut assets_mesh, &assets_shader, &mut commands));

            wall_manager.editing_handles.push(new_editing_handle(
                cursor_ws_w_y,
                &mut assets_mesh,
                &assets_shader,
                &mut commands,
            ));

            //TODO: add a second handle for the end, and it should update its transform as we draw the curve
        }
    }
    // If LMB is pressed, continue the active curve
    else if mouse_button_input.pressed(MouseButton::Left) {
        let (active_curve, preview_entity) = wall_manager.curves.last_mut().unwrap();

        let intersection = cursor_ws;

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

// TODO: doesn't need a mesh! just make the trigger area component to have a volume!
// TODO: for now only one, in the future, need more!
fn new_editing_handle(
    position: Vec3,
    assets_mesh: &mut ResMut<AssetMeshLibrary>,
    assets_shader: &Res<AssetShaderLibrary>,
    commands: &mut Commands,
) -> Entity {
    let shader = assets_shader
        .get_handle_by_name("vertex_color_shader")
        .unwrap();

    // TODO: make bundles out of these
    let debug_preview = commands
        .spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: assets_mesh.add(TriggerAreaPreview::mesh_asset()),
            shader,
            transform: Transform::identity(),
        })
        .insert(TriggerAreaPreview)
        .insert(crate::components::GLDrawMode(gl::LINE_STRIP))
        .id();

    // TODO: make bundles out of these
    // TODO: no need to create a new mesh for UiPrompt every time
    commands
        .spawn()
        .insert_bundle(DrawableMeshBundle {
            mesh: assets_mesh.add(TriggerArea::mesh_asset()),
            shader,
            transform: Transform::from_translation(position),
        })
        .insert(TriggerArea {
            is_mouse_over: false,
            padding: 20,
            debug_preview,
        })
        .id()
}
