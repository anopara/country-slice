use std::collections::HashMap;

use crate::render::shader::ShaderProgram;

use super::{Asset, Handle, HandleId};

pub struct AssetShaderLibrary {
    pub assets: HashMap<Handle<ShaderProgram>, ShaderProgram>,
    by_name: HashMap<String, Handle<ShaderProgram>>,
}

#[allow(dead_code)]
impl AssetShaderLibrary {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn add(&mut self, asset: Asset<ShaderProgram>) -> Handle<ShaderProgram> {
        let id = HandleId::random();
        let handle = Handle::<ShaderProgram>::new(id);
        self.assets.insert(handle, asset.asset);
        if let Some(n) = asset.name {
            self.by_name.insert(n, handle);
        }

        handle
    }

    pub fn get(&self, handle: Handle<ShaderProgram>) -> Option<&ShaderProgram> {
        self.assets.get(&handle)
    }

    pub fn get_mut(&mut self, handle: Handle<ShaderProgram>) -> Option<&mut ShaderProgram> {
        self.assets.get_mut(&handle)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ShaderProgram> {
        let handle = self.by_name.get(&String::from(name))?;
        self.assets.get(handle)
    }

    pub fn get_handle_by_name(&self, name: &str) -> Option<Handle<ShaderProgram>> {
        let handle = self.by_name.get(&String::from(name))?;
        Some(*handle)
    }
}
