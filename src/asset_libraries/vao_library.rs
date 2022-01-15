use std::collections::HashMap;

use crate::render::{mesh::Mesh, shader::ShaderProgram, vao::VAO};

use super::{mesh_library::AssetMeshLibrary, shader_library::AssetShaderLibrary, Handle, HandleId};

pub struct AssetVAOLibrary {
    vao: HashMap<Handle<VAO>, VAO>,
    by_mesh_and_shader: HashMap<(Handle<Mesh>, Handle<ShaderProgram>), Handle<VAO>>,
    // gets every VAO that would be affected by this mesh
    by_mesh: HashMap<Handle<Mesh>, Vec<(Handle<Mesh>, Handle<ShaderProgram>)>>,
}

impl AssetVAOLibrary {
    pub fn new() -> Self {
        Self {
            vao: HashMap::new(),
            by_mesh_and_shader: HashMap::new(),
            by_mesh: HashMap::new(),
        }
    }

    pub fn get(&self, vao_handle: Handle<VAO>) -> Option<&VAO> {
        self.vao.get(&vao_handle)
    }

    pub fn add(
        &mut self,
        mesh_library: &AssetMeshLibrary,
        shader_library: &AssetShaderLibrary,
        mesh_handle: Handle<Mesh>,
        shader_handle: Handle<ShaderProgram>,
    ) -> Result<Handle<VAO>, String> {
        // check if the mesh-shader pair already exists
        if let Some(vao_handle) = self.by_mesh_and_shader.get(&(mesh_handle, shader_handle)) {
            Ok(*vao_handle)
        }
        // if it doesnt, create a new vao
        else {
            let id = HandleId::random();
            let vao_handle = Handle::<VAO>::new(id);

            let vao = self.build_vao(mesh_library, shader_library, mesh_handle, shader_handle)?;
            self.vao.insert(vao_handle, vao);

            self.by_mesh_and_shader
                .insert((mesh_handle, shader_handle), vao_handle);

            if let Some(handles) = self.by_mesh.get_mut(&mesh_handle) {
                handles.push((mesh_handle, shader_handle));
            } else {
                self.by_mesh
                    .insert(mesh_handle, vec![(mesh_handle, shader_handle)]);
            }

            //println!(
            //    "VAO Library: Added {:?} with ShaderID:{:?}",
            //    mesh_handle, shader_handle
            //);

            Ok(vao_handle)
        }
    }

    pub fn has_vao(&self, mesh_handle: Handle<Mesh>) -> bool {
        self.by_mesh.get(&mesh_handle).is_some()
    }

    fn build_vao(
        &self,
        mesh_library: &AssetMeshLibrary,
        shader_library: &AssetShaderLibrary,
        mesh_handle: Handle<Mesh>,
        shader_handle: Handle<ShaderProgram>,
    ) -> Result<VAO, String> {
        let mesh = mesh_library
            .get(mesh_handle)
            .ok_or("Mesh is not in the  mesh library")?;
        if mesh.is_valid() {
            let shader = shader_library
                .get(shader_handle)
                .ok_or("Shader is not in the shader library")?;
            Ok(VAO::new(mesh, &shader.id))
        } else {
            Err(String::from("Mesh is invalid"))
        }
    }

    pub fn rebuild_vao(
        &mut self,
        mesh_library: &AssetMeshLibrary,
        shader_library: &AssetShaderLibrary,
        mesh_handle: Handle<Mesh>,
    ) -> Result<(), String> {
        // get affected VAOs
        let handles = self
            .by_mesh
            .get(&mesh_handle)
            .ok_or("no VAOs match the Mesh Handle")?;

        // rebuild them
        for (mesh_handle, shader_handle) in handles {
            let mesh = mesh_library
                .get(*mesh_handle)
                .expect("Mesh is not in the mesh library");
            if mesh.is_valid() {
                let shader = shader_library
                    .get(*shader_handle)
                    .ok_or("Shader is not in the shader library")?;
                let vao_handle = self
                    .by_mesh_and_shader
                    .get(&(*mesh_handle, *shader_handle))
                    .ok_or("The requested mesh-shader pair doesn't exist")?;
                let vao = self
                    .vao
                    .get_mut(vao_handle)
                    .ok_or("VAO handle is invalid")?;

                vao.rebuild(mesh, &shader.id);
            } else {
                return Err(String::from("VAO couldn't be rebuilt: Mesh is invalid"));
                // TODO: right now if one vao fails, all of them will fail
            }
        }

        Ok(())
    }

    pub fn remove(&mut self, mesh_handle: Handle<Mesh>) {
        // get affected VAOs
        if let Some(affected_h) = self.by_mesh.remove(&mesh_handle) {
            // get affected Mesh-Shader pairs
            for mesh_shader_h in affected_h {
                // if this pair has a valid VAO
                if let Some(vao_h) = self.by_mesh_and_shader.remove(&mesh_shader_h) {
                    // Delete this VAO from memory and remove the handle
                    if let Some(vao) = self.vao.remove(&vao_h) {
                        unsafe {
                            gl::DeleteVertexArrays(1, &vao.id());
                        }
                    }
                }
            }
        }
    }
}
