use crate::{instanced_wall::InstancedWall, shadow_decal::ShadowDecal, utils, CustomMesh};
use bevy::{
    prelude::*,
    render::pipeline::{PipelineDescriptor, RenderPipeline},
};

pub struct UserDrawnCurve {
    pub points: Vec<Vec3>,
    pub debug_mesh_handle: Option<Handle<Mesh>>,
    pub debug_mesh_pipeline: Handle<PipelineDescriptor>, // shader stuff
    pub entity_id: Option<Entity>,
}

impl UserDrawnCurve {
    pub fn new(debug_mesh_pipeline: Handle<PipelineDescriptor>) -> Self {
        Self {
            points: Vec::new(),
            debug_mesh_handle: None,
            debug_mesh_pipeline,
            entity_id: None,
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
                let (tri_mesh, uvs, length) = utils::curve_to_trimesh(&smoothed);
                utils::bevy_mesh_from_trimesh(tri_mesh, uvs, length, bevy_mesh);
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
        mesh.set_attribute("Vertex_Curve_Length", vec![0.0; 3]);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(vec![0, 1, 2])));

        let handle = mesh_assets.add(mesh);
        self.debug_mesh_handle = Some(handle.clone());

        self.entity_id = Some(
            commands
                .spawn_bundle(PbrBundle {
                    mesh: handle,
                    material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        self.debug_mesh_pipeline.clone(),
                    )]),
                    ..Default::default()
                })
                .insert(CustomMesh)
                .id(),
        );
    }
}

pub struct CurveManager {
    pub user_curves: Vec<UserDrawnCurve>,
    pub instanced_walls: Vec<InstancedWall>,
    pub shadow_decals: Vec<ShadowDecal>,
    pub curve_pipeline_handle: Option<Handle<PipelineDescriptor>>,
    pub wall_pipeline_handle: Option<Handle<PipelineDescriptor>>,
    pub shadow_pipeline_handle: Option<Handle<PipelineDescriptor>>,
}

impl CurveManager {
    pub fn new() -> Self {
        Self {
            shadow_decals: Vec::new(),
            user_curves: Vec::new(),
            instanced_walls: Vec::new(),
            curve_pipeline_handle: None,
            wall_pipeline_handle: None,
            shadow_pipeline_handle: None,
        }
    }

    pub fn clear_all(&mut self, commands: &mut Commands) {
        for curve in &self.user_curves {
            if let Some(curve_entity) = curve.entity_id {
                commands.entity(curve_entity).despawn()
            }
        }

        for shadow in &self.shadow_decals {
            commands.entity(shadow.entity_id).despawn()
        }

        for wall in &self.instanced_walls {
            commands.entity(wall.entity_id).despawn()
        }

        self.user_curves = Vec::new();
        self.shadow_decals = Vec::new();
        self.instanced_walls = Vec::new();
    }
}
