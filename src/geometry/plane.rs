use crate::render::mesh::Mesh;

// A square on the XZ plane.
#[derive(Debug, Copy, Clone)]
pub struct Plane {
    /// The total side length of the square.
    pub size: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Plane { size: 1.0 }
    }
}

impl From<Plane> for Mesh {
    fn from(plane: Plane) -> Self {
        let extent = plane.size / 2.0;

        let vertices = [
            ([extent, 0.0, -extent], [0.0, 1.0, 0.0], [1.0, 1.0]),
            ([extent, 0.0, extent], [0.0, 1.0, 0.0], [1.0, 0.0]),
            ([-extent, 0.0, extent], [0.0, 1.0, 0.0], [0.0, 0.0]),
            ([-extent, 0.0, -extent], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        let indices = vec![0, 2, 1, 0, 3, 2];

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        for (position, normal, uv) in vertices.iter() {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }

        let mut mesh = Mesh::new();
        mesh.set_indices(indices);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV, uvs);
        mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0]; vertices.len()]);
        mesh
    }
}
