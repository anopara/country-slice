use crate::curve::Curve;
use bevy::{prelude::*, render::mesh::VertexAttributeValues};

pub fn bevy_mesh_from_trimesh(
    tri_mesh: tri_mesh::mesh::Mesh,
    uvs: Vec<[f32; 2]>,
    curve_length: f32,
    bevy_mesh: &mut Mesh,
) {
    let vert_count = tri_mesh.vertex_iter().count();

    let positions: Vec<[f32; 3]> = tri_mesh
        .positions_buffer_f32()
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let normals: Vec<[f32; 3]> = tri_mesh
        .normals_buffer_f32()
        .chunks(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let indices = tri_mesh.indices_buffer();
    //let mut other_side = indices.clone();
    //other_side.reverse();
    //indices.extend(&other_side);

    //TODO: reverse normals on the other side (might need double vertices then)

    // MAKE SURE THE ATTRIBUTES ARE THE SAME WHEN DEBUG MESH IS INIT IN `UserDrawnCurve`
    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals); //vec![[1.0, 0.0, 0.0]; vert_count]);
    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
    bevy_mesh.set_attribute("Vertex_Curve_Length", vec![curve_length; vert_count]);
    bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
}

pub fn curve_to_trimesh(points: &[Vec3]) -> (tri_mesh::mesh::Mesh, Vec<[f32; 2]>, f32) {
    let curve_positions = points;

    let mut indices: Vec<u32> = Vec::new();
    let mut positions: Vec<f64> = Vec::new();

    let temp_curve = Curve::new(points.to_vec());
    let curve_u = temp_curve.points_u;
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    for quad_index in 0..(curve_positions.len() - 1) {
        let start_point = curve_positions[quad_index];
        let end_point = curve_positions[quad_index + 1];

        let vert_index_start = (positions.len() / 3) as u32;

        positions.extend(&vec![
            // start vertex
            start_point[0] as f64,
            start_point[1] as f64,
            start_point[2] as f64,
            // offset up
            start_point[0] as f64,
            start_point[1] as f64 + 1.0,
            start_point[2] as f64,
            // end vertex
            end_point[0] as f64,
            end_point[1] as f64,
            end_point[2] as f64,
            // offset up
            end_point[0] as f64,
            end_point[1] as f64 + 1.0,
            end_point[2] as f64,
        ]);

        uvs.extend(&vec![
            // start vertex
            [curve_u[quad_index], 0.0],
            // offset up
            [curve_u[quad_index], 1.0],
            // end vertex
            [curve_u[quad_index + 1], 0.0],
            // offset up
            [curve_u[quad_index + 1], 1.0],
        ]);

        indices.extend(
            &([0, 1, 2, 1, 3, 2]
                .iter()
                .map(|i| i + vert_index_start)
                .collect::<Vec<_>>()),
        )
    }

    let mesh = tri_mesh::MeshBuilder::new()
        .with_indices(indices)
        .with_positions(positions)
        .build()
        .unwrap();

    (mesh, uvs, temp_curve.length)
}

pub fn smooth_points(points: &Vec<Vec3>, smoothing_steps: usize) -> Vec<Vec3> {
    let mut total_smoothed = points.clone();
    for _ in 0..smoothing_steps {
        let mut current_iter_smooth = total_smoothed.clone();
        for (i, current_pos) in total_smoothed.iter().enumerate() {
            if let (Some(prev_pos), Some(next_pos)) =
                (total_smoothed.get(i - 1), total_smoothed.get(i + 1))
            {
                let avg: Vec3 = (*prev_pos + *next_pos) / 2.0;
                current_iter_smooth[i] = *current_pos + (avg - *current_pos) * 0.5;
            }
        }
        total_smoothed = current_iter_smooth;
    }

    total_smoothed.to_vec()
}

pub struct MeshBuffer {
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub colors: Vec<[f32; 4]>,
    pub tex_coord: Vec<[f32; 2]>,
}

pub fn load_gltf_as_mesh_buffer(path: &str) -> MeshBuffer {
    let mut out = MeshBuffer {
        indices: Vec::new(),
        positions: Vec::new(),
        normals: Vec::new(),
        tangents: Vec::new(),
        colors: Vec::new(),
        tex_coord: Vec::new(),
    };
    let (gltf, buffers, _) = gltf::import(path).unwrap();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(vertex_attribute) =
                reader.read_colors(0).map(|v| v.into_rgba_f32().collect())
            {
                out.colors = vertex_attribute;
            }

            if let Some(vertex_attribute) = reader.read_positions().map(|v| v.collect()) {
                out.positions = vertex_attribute;
            }

            if let Some(vertex_attribute) = reader.read_normals().map(|v| v.collect()) {
                out.normals = vertex_attribute;
            }

            if let Some(vertex_attribute) = reader.read_tangents().map(|v| v.collect()) {
                out.tangents = vertex_attribute;
            }

            if let Some(vertex_attribute) =
                reader.read_tex_coords(0).map(|v| v.into_f32().collect())
            {
                out.tex_coord = vertex_attribute;
            }

            if let Some(indices) = reader.read_indices().map(|v| v.into_u32().collect()) {
                out.indices = indices;
            };
        }
    }
    out
}

pub fn load_gltf_as_bevy_mesh_w_vertex_color(path: &str) -> Mesh {
    let mut bevy_mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
    let (gltf, buffers, _) = gltf::import(path).unwrap();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(vertex_attribute) = reader
                .read_colors(0)
                .map(|v| VertexAttributeValues::Float4(v.into_rgba_f32().collect()))
            {
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_positions()
                .map(|v| VertexAttributeValues::Float3(v.collect()))
            {
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_normals()
                .map(|v| VertexAttributeValues::Float3(v.collect()))
            {
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_tangents()
                .map(|v| VertexAttributeValues::Float4(v.collect()))
            {
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_TANGENT, vertex_attribute);
            }

            if let Some(vertex_attribute) = reader
                .read_tex_coords(0)
                .map(|v| VertexAttributeValues::Float2(v.into_f32().collect()))
            {
                bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vertex_attribute);
            }

            if let Some(indices) = reader.read_indices() {
                bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
                    indices.into_u32().collect(),
                )));
            };
        }
    }
    bevy_mesh
}
