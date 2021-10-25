use crate::{utils, CustomMesh};
use bevy::{prelude::*, render::pipeline::PipelineDescriptor};
use std::collections::HashMap;

pub struct UserDrawnCurve {
    pub points: Vec<Vec3>,
    pub debug_mesh_handle: Option<Handle<Mesh>>,
}

impl UserDrawnCurve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            debug_mesh_handle: None,
        }
    }

    pub fn update_debug_mesh(
        &mut self,
        mut mesh_assets: ResMut<Assets<Mesh>>,
        materials: ResMut<Assets<StandardMaterial>>,
        commands: Commands,
    ) {
        if let Some(mesh_handle) = self.debug_mesh_handle.as_ref() {
            if let Some(bevy_mesh) = mesh_assets.get_mut(mesh_handle) {
                let smoothed = utils::smooth_points(&self.points, 50);
                let tri_mesh = utils::curve_to_trimesh(&smoothed);
                utils::bevy_mesh_from_trimesh(tri_mesh, bevy_mesh);
            } else {
                warn!("UserDrawnCurve: bevy mesh doesn't exist");
            }
        } else {
            self.create_debug_mesh(mesh_assets, materials, commands);
        }
    }

    fn create_debug_mesh(
        &mut self,
        mut mesh_assets: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut commands: Commands,
    ) {
        let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

        // Create a single triangle somewhere under the level, the data will be overwritten next tick anyway
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0.0, -100.0, 0.0], [1.0, -100.0, 0.0], [0.0, -101.0, 0.0]],
        );
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, -1.0]; 3]);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; 3]);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(vec![0, 1, 2])));

        let handle = mesh_assets.add(mesh);
        self.debug_mesh_handle = Some(handle.clone());

        commands
            .spawn_bundle(PbrBundle {
                mesh: handle,
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                ..Default::default()
            })
            .insert(CustomMesh);
    }
}

pub struct CurveManager {
    pub user_curves: Vec<UserDrawnCurve>,
    pub walls: Vec<Vec<Entity>>,
    pub brick_mesh_handle: Option<Handle<Mesh>>,
    pub brick_pipeline_handle: Option<Handle<PipelineDescriptor>>,
}

impl CurveManager {
    pub fn new() -> Self {
        Self {
            user_curves: Vec::new(),
            walls: Vec::new(),
            brick_mesh_handle: None,
            brick_pipeline_handle: None,
        }
    }

    /*
        pub fn smooth_positions(&self) -> Vec<Vec3> {
            let points_per_segment = 10;
            let smoothing_steps = 50;

            // resample curve
            let mut resampled: Vec<Vec3> = Vec::new();
            for (i, current_pos) in self.point_positions.iter().enumerate() {
                if let Some(next_pos) = self.point_positions.get(i + 1) {
                    let dir = *next_pos - *current_pos;
                    resampled.extend(
                        &(0..points_per_segment)
                            .map(|s| *current_pos + dir * (s as f32 / points_per_segment as f32))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    // if last point, just add
                    resampled.push(*current_pos);
                }
            }

            // smooth
            utils::smooth_points(resampled, smoothing_steps)
        }
    */
    /*
    fn to_vertices(&self) -> Vec<[f32; 3]> {
        let mut one_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1] + 1.0, p[2]], [p[0], p[1], p[2]]])
            .flatten()
            .collect();
        let mut other_side: Vec<[f32; 3]> = self
            .point_positions
            .iter()
            .map(|p| vec![[p[0], p[1], p[2]], [p[0], p[1] + 1.0, p[2]]])
            .flatten()
            .collect();
        other_side.reverse();
        one_side.extend(&other_side);
        one_side
    }
    */
}
