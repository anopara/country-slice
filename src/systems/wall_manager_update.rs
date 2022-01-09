use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButton, Input};

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle},
    components::{drawable::DrawableMeshBundle, transform::Transform},
    geometry::{instanced_wall::*, shadow_decal::ShadowDecal, wall_constructor::*},
    render::mesh::Mesh,
    resources::WallManager,
    systems::mode_manager::ActiveCurve,
};

use super::mode_manager::Mode;

pub fn walls_update(
    mode: Res<Mode>,

    mut wall_manager: ResMut<WallManager>,
    mut query: Query<&mut InstancedWall>,
    mut query3: Query<(&mut ShadowDecal, &mut Handle<Mesh>)>,
    mut assets_mesh: ResMut<AssetMeshLibrary>,
    assets_shader: Res<AssetShaderLibrary>,
    mut commands: Commands,
) {
    puffin::profile_function!();

    match &*mode {
        Mode::None => (),
        Mode::StartNewCurve => (),
        Mode::DrawingCurve(active_curve, _) => {
            let active_curve_index = match active_curve {
                ActiveCurve::Last => wall_manager.curves.len() - 1,
                ActiveCurve::Index(index) => *index,
            };

            let (active_curve, _) = wall_manager.curves.get(active_curve_index).unwrap();

            if active_curve.points.len() < 2 {
                return;
            }

            // Calculate brick transforms
            let curve = {
                puffin::profile_scope!("curve");
                active_curve.clone().smooth(50).resample(0.2)
            };

            {
                puffin::profile_scope!("construct wall");
                let bricks = WallConstructor::from_curve(&curve);

                if bricks.is_empty() {
                    log::warn!("WallConstructor returned empty wall");
                }

                if let Some(wall_entity) = wall_manager.walls.get(active_curve_index) {
                    // update the wall
                    let mut wall_component = query.get_mut(*wall_entity).unwrap();
                    wall_component.update(curve.length, bricks);
                } else {
                    //create a wall
                    log::info!("creating wall..");
                    wall_manager.walls.push(create_wall(
                        curve.length,
                        bricks,
                        &assets_mesh,
                        &assets_shader,
                        &mut commands,
                    ));
                }
            }

            {
                puffin::profile_scope!("shadow decal");
                if let Some(shadow_entity) = wall_manager.shadows.get(active_curve_index) {
                    let (_shadow_component, mesh_handle) = query3.get_mut(*shadow_entity).unwrap();
                    let mesh = assets_mesh.get_mut(*mesh_handle).unwrap();
                    ShadowDecal::update(&curve, mesh);
                } else {
                    wall_manager.shadows.push(ShadowDecal::new(
                        &curve,
                        &mut assets_mesh,
                        &assets_shader,
                        &mut commands,
                    ));
                }
            }
        }
        Mode::EditingCurve(_) => todo!(),
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
