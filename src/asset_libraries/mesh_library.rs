use std::collections::HashMap;

use crate::render::mesh::Mesh;

use super::{Asset, Handle, HandleId};

pub struct AssetMeshLibrary {
    mesh: HashMap<Handle<Mesh>, Mesh>,
    by_name: HashMap<String, Handle<Mesh>>,

    // these meshes have changed and need their associated vaos rebuilt
    pub marked_as_dirty: Vec<Handle<Mesh>>,
}

#[allow(dead_code)]
impl AssetMeshLibrary {
    pub fn new() -> Self {
        Self {
            mesh: HashMap::new(),
            by_name: HashMap::new(),
            marked_as_dirty: Vec::new(),
        }
    }

    pub fn add(&mut self, asset: Asset<Mesh>) -> Handle<Mesh> {
        let id = HandleId::random();
        let mesh_handle = Handle::<Mesh>::new(id);

        // add mesh data
        self.mesh.insert(mesh_handle, asset.asset);
        if let Some(n) = asset.name {
            self.by_name.insert(n, mesh_handle);
        }

        mesh_handle
    }

    pub fn get(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.mesh.get(&handle.into())
    }

    pub fn get_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        self.marked_as_dirty.push(handle);
        self.mesh.get_mut(&handle.into())
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Mesh> {
        let id = self.by_name.get(&String::from(name))?;
        self.mesh.get(id)
    }

    pub fn get_handle_by_name(&self, name: &str) -> Option<Handle<Mesh>> {
        self.by_name.get(&String::from(name)).map(|h| *h)
    }

    pub fn is_dirty(&self, handle: Handle<Mesh>) -> bool {
        self.marked_as_dirty.contains(&handle)
    }
}
