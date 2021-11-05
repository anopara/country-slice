use crate::{utils, wall_constructor::Brick, CustomMesh};
use bevy::{
    prelude::*,
    render::pipeline::{PipelineDescriptor, RenderPipeline},
};
use utils::MeshBuffer;

pub struct InstancedWall {
    bevy_mesh_handle: Handle<Mesh>, // mesh of the whole wall
    mesh_buffer: MeshBuffer,        // pre-loaded mesh of a single brick
    pub entity_id: Entity,
}

impl InstancedWall {
    pub fn new(
        bricks: Vec<Brick>,
        mesh_assets: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        render_pipeline: Handle<PipelineDescriptor>,
        commands: &mut Commands,
    ) -> Self {
        // create a mesh
        let mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        let mesh_buffer = utils::load_gltf_as_mesh_buffer("assets/meshes/brick.glb");
        let mut out = Self {
            bevy_mesh_handle: mesh_assets.add(mesh),
            mesh_buffer,
            entity_id: Entity::new(0), // garbage, so we can init, this is overwritten right after
        };
        out.update(bricks, mesh_assets);

        out.entity_id = commands
            .spawn_bundle(PbrBundle {
                mesh: out.bevy_mesh_handle.clone(),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: None,
                    roughness: 0.9,
                    metallic: 0.0,
                    metallic_roughness_texture: None,
                    reflectance: 0.1,
                    normal_map: None,
                    double_sided: false,
                    occlusion_texture: None,
                    emissive: Color::BLACK,
                    emissive_texture: None,
                    unlit: false,
                }),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    render_pipeline,
                )]),
                ..Default::default()
            })
            .insert(CustomMesh)
            .id();

        out
    }

    pub fn update(&mut self, bricks: Vec<Brick>, mesh_assets: &mut ResMut<Assets<Mesh>>) {
        if let Some(bevy_mesh) = mesh_assets.get_mut(self.bevy_mesh_handle.clone()) {
            let mut positions: Vec<[f32; 3]> = Vec::new();
            let mut normals: Vec<[f32; 3]> = Vec::new();
            let mut uvs: Vec<[f32; 2]> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            let mut instance_ids: Vec<u32> = Vec::new();
            let mut curve_uv: Vec<[f32; 2]> = Vec::new();
            let mut sin_offset_per_row: Vec<f32> = Vec::new();

            let mesh_vert_count = self.mesh_buffer.indices.len();

            for (i, brick) in bricks.iter().enumerate() {
                let from_os_to_ws = Mat4::from_scale_rotation_translation(
                    brick.scale,
                    brick.rotation,
                    brick.position,
                );

                // Record WS positions
                positions.extend(
                    &self
                        .mesh_buffer
                        .positions
                        .clone()
                        .iter()
                        .map(|p| {
                            let v = from_os_to_ws.transform_point3(Vec3::from_slice_unaligned(p));
                            [v.x, v.y, v.z]
                        })
                        .collect::<Vec<_>>(),
                );

                // Record WS normals
                // Technically scaling can affect the normals, unless uniform (TODO: take non-uniform scaling into account)
                normals.extend(
                    &self
                        .mesh_buffer
                        .normals
                        .clone()
                        .iter()
                        .map(|n| {
                            let v = brick.rotation.mul_vec3(Vec3::from_slice_unaligned(n));
                            [v.x, v.y, v.z]
                        })
                        .collect::<Vec<_>>(),
                );

                uvs.extend(&self.mesh_buffer.tex_coord);

                indices.extend(
                    &self
                        .mesh_buffer
                        .indices
                        .clone()
                        .iter()
                        .map(|ind| ind + ((mesh_vert_count * i) as u32))
                        .collect::<Vec<_>>(),
                );

                instance_ids.extend(&vec![i as u32; mesh_vert_count]);

                curve_uv.extend(
                    self.mesh_buffer
                        .positions
                        .iter()
                        .map(|p| {
                            let bbx_pos = Vec3::from_slice_unaligned(p) + Vec3::splat(0.5);
                            let curve_uv_pos =
                                brick.pivot_uv + Vec2::new(bbx_pos.x, bbx_pos.y) * brick.bounds_uv;
                            [curve_uv_pos.x, curve_uv_pos.y]
                        })
                        .collect::<Vec<_>>(),
                );

                // TODO: needs better name.. its like, in-between brock rows
                sin_offset_per_row.extend(
                    self.mesh_buffer
                        .positions
                        .iter()
                        .map(|p| {
                            // if it's a bottom of the brick, the bottom should have ID = row ID
                            // if its a top of the brick, it should have same offset as the bottom fo the row ID + 1
                            let bbx_pos = Vec3::from_slice_unaligned(p) + Vec3::splat(0.5);
                            if bbx_pos.y < 0.5 {
                                brick.row_id_bottom as f32 / (brick.row_count as f32)
                            } else {
                                // if it's the last row, than brick deformation can be random on the upper part of the brick
                                if brick.row_id_bottom == brick.row_count {
                                    fastrand::Rng::with_seed(i as u64).f32()
                                } else {
                                    (brick.row_id_top as f32) / (brick.row_count as f32)
                                }
                            }
                        })
                        .collect::<Vec<_>>(),
                );
            }

            // populate bevy mesh
            bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            bevy_mesh.set_attribute("Instance_Id", instance_ids);
            bevy_mesh.set_attribute("Curve_Uv_Pos", curve_uv);
            bevy_mesh.set_attribute("Sin_Offset_Per_Row", sin_offset_per_row);
            bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        }
    }
}
