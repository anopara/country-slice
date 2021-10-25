use bevy::{prelude::*, render::mesh::VertexAttributeValues};

pub fn bevy_mesh_from_trimesh(tri_mesh: tri_mesh::mesh::Mesh, bevy_mesh: &mut Mesh) {
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
    let mut indices = tri_mesh.indices_buffer();
    let mut other_side = indices.clone();
    other_side.reverse();
    indices.extend(&other_side);

    //TODO: reverse normals on the other side (might need double vertices then)

    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals); //vec![[1.0, 0.0, 0.0]; vert_count]);
    bevy_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.0]; vert_count]);
    bevy_mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
}

pub fn curve_to_trimesh(points: &[Vec3]) -> tri_mesh::mesh::Mesh {
    let curve_positions = points;

    let mut indices: Vec<u32> = Vec::new();
    let mut positions: Vec<f64> = Vec::new();
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

    mesh
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
