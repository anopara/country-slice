use bevy_app::EventReader;
use bevy_ecs::prelude::*;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{drawable::DrawableMeshBundle, transform::Transform},
    geometry::{instanced_wall::*, shadow_decal::ShadowDecal, wall_constructor::*},
    render::mesh::Mesh,
    resources::{events::CurveChangedEvent, WallManager},
    systems::mode_manager::{BrushMode, EraseLayer},
};

pub fn walls_update(
    _mode: Res<BrushMode>,
    mut ev_curve_changed: EventReader<CurveChangedEvent>,

    mut wall_manager: ResMut<WallManager>,
    mut query: Query<&mut InstancedWall>,
    mut query3: Query<(&mut ShadowDecal, &mut Handle<Mesh>)>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
    mut commands: Commands,
) {
    if !matches!(*_mode, BrushMode::Wall) && !matches!(*_mode, BrushMode::Eraser(EraseLayer::All)) {
        return;
    }

    puffin::profile_function!();

    for ev in ev_curve_changed.iter() {
        let changed_wall = wall_manager.get_mut(ev.curve_index).expect(&format!(
            "Wall construction failed: couldn't get Wall index {}",
            ev.curve_index,
        ));

        if changed_wall.curve.points.len() < 2 {
            continue;
        }

        // Calculate brick transforms
        {
            puffin::profile_scope!("construct wall");
            let bricks = WallConstructor::from_curve(&changed_wall.curve);

            if bricks.is_empty() {
                log::warn!("WallConstructor returned empty wall");
            }

            if let Some(wall_entity) = changed_wall.wall_entity {
                // update the wall
                let mut wall_component = query.get_mut(wall_entity).unwrap();
                wall_component.update(changed_wall.curve.length, bricks);
            } else {
                //create a wall
                log::info!("creating wall..");

                changed_wall.wall_entity = Some(create_wall(
                    changed_wall.curve.length,
                    bricks,
                    &assets_mesh,
                    &assets_shader,
                    &mut commands,
                ));
            }
        }

        {
            puffin::profile_scope!("shadow decal");
            if let Some(shadow_entity) = changed_wall.shadow {
                let (_shadow_component, mesh_handle) = query3.get_mut(shadow_entity).unwrap();
                let mesh = assets_mesh.get_mut(*mesh_handle).unwrap();
                ShadowDecal::update(&changed_wall.curve, mesh);
            } else {
                changed_wall.shadow = Some(ShadowDecal::new(
                    &changed_wall.curve,
                    &mut assets_mesh,
                    &assets_shader,
                    &mut commands,
                ));
            }
        }
    }
}

fn create_wall(
    curve_length: f32,
    bricks: Vec<Brick>,
    assets_mesh: &ResMut<AssetMeshLibrary>,
    assets_shader: &Res<AssetShaderLibrary>,
    commands: &mut Commands,
) -> Entity {
    let wall_component = InstancedWall::from(curve_length, bricks);
    let brick_mesh_handle = assets_mesh.get_handle_by_name("brick").unwrap();

    commands
        .spawn()
        .insert(wall_component)
        .insert_bundle(DrawableMeshBundle {
            mesh: brick_mesh_handle,
            shader: assets_shader
                .get_handle_by_name("instanced_wall_shader")
                .unwrap(),
            transform: Transform::identity(),
        })
        .id()
}
