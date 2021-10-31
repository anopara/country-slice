use crate::curve::Curve;
use bevy::{
    prelude::*,
    render::pipeline::{PipelineDescriptor, RenderPipeline},
};

// TODO:
// another side
// caps

const SHADOW_WIDTH: f32 = 0.3;

pub struct ShadowDecal {
    mesh_handle: Handle<Mesh>,
    entity_id: Entity,
    // id is used as a HACKy way to Z-sort through shadow meshes, so new meshes are always on top
    id: usize,
}

impl ShadowDecal {
    pub fn new(
        curve: &Curve,
        mesh_assets: &mut ResMut<Assets<Mesh>>,
        render_pipeline: Handle<PipelineDescriptor>,
        commands: &mut Commands,
        shadow_id: usize,
    ) -> Self {
        // create a mesh
        let mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        let mut out = Self {
            mesh_handle: mesh_assets.add(mesh),
            entity_id: Entity::new(0), // garbage, just so we can init the struct, this is overwritten right after
            id: shadow_id,
        };

        out.update(curve, mesh_assets)
            .expect("Shadow Decal: couldn't update shadow mesh");

        out.entity_id = commands
            .spawn_bundle(PbrBundle {
                mesh: out.mesh_handle.clone(),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    render_pipeline,
                )]),
                visible: Visible {
                    is_transparent: true,
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();

        out
    }

    pub fn update(&self, curve: &Curve, mesh_assets: &mut ResMut<Assets<Mesh>>) -> Option<()> {
        let offset_from_ground = 0.001 * (self.id as f32);

        let bevy_mesh = mesh_assets.get_mut(self.mesh_handle.clone())?;

        let curve_pts = &curve.points;
        let offset_pts: Vec<(Vec3, Vec3)> = curve_pts
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let (this, next) = if let Some(next) = curve_pts.get(idx + 1) {
                    (p, next)
                } else {
                    (
                        curve_pts.get(idx - 1).expect(
                            "ShadowDecal: there was not pervious point to constuct tangent",
                        ),
                        p,
                    )
                };

                let tangent = (*next - *this).normalize();

                (
                    tangent.cross(-Vec3::Y) * SHADOW_WIDTH,
                    tangent.cross(Vec3::Y) * SHADOW_WIDTH,
                )
            })
            .collect();

        // create a mesh
        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();

        for quad_index in 0..(curve_pts.len() - 1) {
            let start = curve_pts[quad_index];
            let l_start = start + offset_pts[quad_index].0;
            let r_start = start + offset_pts[quad_index].1;
            let end = curve_pts[quad_index + 1];
            let l_end = end + offset_pts[quad_index + 1].0;
            let r_end = end + offset_pts[quad_index + 1].1;

            positions.extend(&[
                //start vertex
                [start[0], start[1] + offset_from_ground, start[2]],
                // end vertex
                [end[0], end[1] + offset_from_ground, end[2]],
                // start vertex + left offset
                [l_start[0], l_start[1] + offset_from_ground, l_start[2]],
                // end vertex + left offset
                [l_end[0], l_end[1] + offset_from_ground, l_end[2]],
                // start vertex + right offset
                [r_start[0], r_start[1] + offset_from_ground, r_start[2]],
                // end vertex + right offset
                [r_end[0], r_end[1] + offset_from_ground, r_end[2]],
            ]);

            uvs.extend(&vec![
                // start vertex
                [0.0, 0.0],
                // end vertex
                [1.0, 0.0],
                // left offset
                [0.0, 1.0],
                // left offset
                [1.0, 1.0],
                // right offset
                [0.0, 1.0],
                // right offset
                [1.0, 1.0],
            ]);

            indices.extend(
                &([0, 1, 2, 1, 3, 2, 0, 4, 1, 4, 5, 1]
                    .iter()
                    .map(|i| i + (quad_index * 6) as u32)
                    .collect::<Vec<_>>()),
            )
        }

        let normals = vec![[0.0, 1.0, 0.0]; positions.len()];

        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        Some(())
    }
}
