use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Asset},
    components::{
        drawable::{DrawableMeshBundle, TransparencyPass},
        transform::Transform,
    },
    render::mesh::Mesh,
};

use super::curve::Curve;

const OFFSET_FROM_GROUND: f32 = 0.001;
const SHADOW_WIDTH: f32 = 0.5;
const SHADOW_CAP_STEPS: usize = 10;

pub struct ShadowDecal;

impl ShadowDecal {
    pub fn new(
        curve: &Curve,
        mesh_assets: &mut ResMut<AssetMeshLibrary>,
        assets_shader: &Res<AssetShaderLibrary>,
        commands: &mut Commands,
    ) -> Entity {
        let mut mesh = Mesh::new();

        Self::update(curve, &mut mesh).expect("Shadow Decal: couldn't update shadow mesh");

        commands
            .spawn_bundle(DrawableMeshBundle {
                mesh: mesh_assets.add(Asset::new(mesh).name("shadow decal")),
                shader: assets_shader
                    .get_handle_by_name("shadow_shader")
                    .expect("Missing shadow shader"),
                transform: Transform::identity(),
            })
            .insert(ShadowDecal)
            .insert(TransparencyPass)
            .id()
    }

    pub fn update(curve: &Curve, mesh: &mut Mesh) -> Option<()> {
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
                            "ShadowDecal: there was not pervious point to construct tangent",
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

        // Trim the first and last point, because shadow caps will sit on top of them (this assumed uniformly sampled curve)
        for quad_index in 1..(curve_pts.len() - 2) {
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

        let caps = if curve_pts.len() == 2 {
            // If we only have 2 points, place two caps inbetween those two points
            let start = (curve_pts[1] + curve_pts[0]) / 2.0;
            let end = start;
            let t_start = (curve_pts[0] - curve_pts[1]).normalize();
            let t_end = (curve_pts[1] - curve_pts[0]).normalize();
            vec![(start, t_start), (end, t_end)]
        } else {
            // Trimming point 0 and last one, because caps will go over them
            let start_index = 1;
            let end_index = curve_pts.len() - 2;
            let start = curve_pts[start_index];
            let end = curve_pts[end_index];
            let t_start = (curve_pts[start_index + 1] - curve_pts[start_index]).normalize();
            let t_end = -(curve_pts[end_index + 1] - curve_pts[end_index]).normalize();
            vec![(start, t_start), (end, t_end)]
        };

        for (position, tangent) in caps {
            add_a_cap(position, tangent, &mut indices, &mut positions, &mut uvs);
        }

        let normals = vec![[0.0, 1.0, 0.0]; positions.len()];

        mesh.set_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[1.0, 0.0, 0.0]; positions.len()],
        );
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV, uvs);

        mesh.set_indices(indices);

        Some(())
    }
}

// TODO: make it a beveled square, like a brick, and not a circle
// can pre-build the mesh in DSS
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
