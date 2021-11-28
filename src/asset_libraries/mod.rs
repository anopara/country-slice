use std::marker::PhantomData;

use crate::render::{mesh::Mesh, shader::ShaderProgram};

pub mod mesh_library;
pub mod shader_library;
pub mod vao_library;

// For ease of passing an optional name to asset manager
pub struct Asset<T> {
    asset: T,
    name: Option<String>,
}

impl<T> Asset<T> {
    pub fn new(asset: T) -> Self {
        Self { asset, name: None }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(String::from(name));
        self
    }
}

impl From<Mesh> for Asset<Mesh> {
    fn from(value: Mesh) -> Self {
        Self::new(value)
    }
}

impl From<ShaderProgram> for Asset<ShaderProgram> {
    fn from(value: ShaderProgram) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct HandleId(u64);

impl HandleId {
    pub fn random() -> Self {
        HandleId(fastrand::u64(..))
    }
}

impl<T> From<Handle<T>> for HandleId {
    fn from(value: Handle<T>) -> Self {
        value.id
    }
}

pub struct Handle<T> {
    id: HandleId,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Handle<T> {
    pub fn new(id: HandleId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.id))
    }
}

// --------------------------------------------------------------------------------------

/*
pub struct Assets<T> {
    assets: HashMap<HandleId, T>,
    by_name: HashMap<String, HandleId>,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn add_w_specific_handle(&mut self, asset: T, handle_id: HandleId) {
        self.assets.insert(handle_id, asset);
    }

    pub fn add(&mut self, asset: Asset<T>) -> Handle<T> {
        let id = HandleId::random();
        self.assets.insert(id, asset.asset);
        if let Some(n) = asset.name {
            self.by_name.insert(n, id);
        }
        Handle::<T>::new(id)
    }

    pub fn get<H: Into<HandleId>>(&self, handle: H) -> Option<&T> {
        self.assets.get(&handle.into())
    }

    pub fn get_mut<H: Into<HandleId>>(&mut self, handle: H) -> Option<&mut T> {
        self.assets.get_mut(&handle.into())
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        let id = self.by_name.get(&String::from(name))?;
        self.assets.get(id)
    }

    pub fn get_handle_from_name(&self, name: &str) -> Option<Handle<T>> {
        let id = self.by_name.get(&String::from(name))?;
        Some(Handle::<T>::new(*id))
    }
}
*/
