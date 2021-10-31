use crate::curve::Curve;
use bevy::{
    prelude::*,
    render::pipeline::{PipelineDescriptor, RenderPipeline},
};

// TODO:
// another side
// caps

const OFFSET_FROM_GROUND: f32 = 0.001;
const SHADOW_WIDTH: f32 = 0.3;
const SHADOW_CAP_STEPS: usize = 10;

pub struct ShadowDecal {
    mesh_handle: Handle<Mesh>,
    entity_id: Entity,
}

impl ShadowDecal {
    pub fn new(
        curve: &Curve,
        mesh_assets: &mut ResMut<Assets<Mesh>>,
        render_pipeline: Handle<PipelineDescriptor>,
        commands: &mut Commands,
    ) -> Self {
        // create a mesh
        let mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        let mut out = Self {
            mesh_handle: mesh_assets.add(mesh),
            entity_id: Entity::new(0), // garbage, just so we can init the struct, this is overwritten right after
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
        let bevy_mesh = mesh_assets.get_mut(self.mesh_handle.clone())?;

        let curve_pts = &curve.points;
        let offset_pts: Vec<Vec3> = curve_pts
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

                tangent.cross(Vec3::Y) * SHADOW_WIDTH
            })
            .collect();

        // create a mesh
        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();

        for quad_index in 0..(curve_pts.len() - 1) {
            let start = curve_pts[quad_index];
            let l_start = start - offset_pts[quad_index];
            let r_start = start + offset_pts[quad_index];
            let end = curve_pts[quad_index + 1];
            let l_end = end - offset_pts[quad_index + 1];
            let r_end = end + offset_pts[quad_index + 1];

            indices.extend(
                &([0, 1, 2, 1, 3, 2, 0, 4, 1, 4, 5, 1]
                    .iter()
                    .map(|i| i + positions.len() as u32)
                    .collect::<Vec<_>>()),
            );

            positions.extend(&[
                //start vertex
                [start[0], start[1] + OFFSET_FROM_GROUND, start[2]],
                // end vertex
                [end[0], end[1] + OFFSET_FROM_GROUND, end[2]],
                // start vertex + left offset
                [l_start[0], l_start[1] + OFFSET_FROM_GROUND, l_start[2]],
                // end vertex + left offset
                [l_end[0], l_end[1] + OFFSET_FROM_GROUND, l_end[2]],
                // start vertex + right offset
                [r_start[0], r_start[1] + OFFSET_FROM_GROUND, r_start[2]],
                // end vertex + right offset
                [r_end[0], r_end[1] + OFFSET_FROM_GROUND, r_end[2]],
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
        }

        // add an end cap
        let last_pt = curve_pts.len() - 1;
        add_a_cap(
            curve_pts[last_pt],
            -(curve_pts[last_pt] - curve_pts[last_pt - 1]).normalize(),
            &mut indices,
            &mut positions,
            &mut uvs,
        );

        // add a start cap
        add_a_cap(
            curve_pts[0],
            (curve_pts[1] - curve_pts[0]).normalize(),
            &mut indices,
            &mut positions,
            &mut uvs,
        );

        let normals = vec![[0.0, 1.0, 0.0]; positions.len()];

        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        Some(())
    }
}

// TODO: make it a beveled square, like a brick, and not a circle
// pre-build mesh in DSS
fn add_a_cap(
    position: Vec3,
    tangent: Vec3,
    indices: &mut Vec<u32>,
    positions: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
) {
    let offset_dir = tangent.cross(Vec3::Y) * SHADOW_WIDTH;

    let cap_pos: Vec<[f32; 3]> = (0..SHADOW_CAP_STEPS)
        .map(|s| {
            let t = (s as f32) / (SHADOW_CAP_STEPS as f32 - 1.0);
            let rot = Quat::from_rotation_y(-3.14 * t);
            let p = position + rot.mul_vec3(offset_dir);
            [p[0], p[1], p[2]]
        })
        .collect();

    let new_indices: Vec<u32> = (0..SHADOW_CAP_STEPS).filter_map(|s|
    // if its not the last point
    if s != SHADOW_CAP_STEPS-1 {
        Some([(s+1) as u32, 0, (s+2) as u32])
    } else {
        None
    }).flatten().collect();

    let cap_uvs: Vec<[f32; 2]> = (0..SHADOW_CAP_STEPS)
        .map(|s| [(s as f32) / (SHADOW_CAP_STEPS as f32 - 1.0), 1.0])
        .collect();

    // Add indices
    indices.extend(
        &new_indices
            .iter()
            .map(|i| i + (positions.len() as u32))
            .collect::<Vec<_>>(),
    );

    // Add starting point
    positions.push([position[0], position[1] + OFFSET_FROM_GROUND, position[2]]);
    uvs.push([0.0, 0.0]);

    // Add the cap
    positions.extend(&cap_pos);
    uvs.extend(&cap_uvs);
}
