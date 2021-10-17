use bevy::{prelude::*, render::mesh::VertexAttributeValues};

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
