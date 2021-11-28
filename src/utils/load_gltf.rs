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
