use crate::{asset_libraries::Handle, render::mesh::Mesh};

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref MESHES_TO_DELETE: Mutex<Vec<Handle<Mesh>>> = Mutex::new(vec![]);
}

// When this component is out of scope, the associated mesh will be deleted
pub struct TransientMesh(pub Handle<Mesh>);

impl Drop for TransientMesh {
    fn drop(&mut self) {
        log::debug!("Transient mesh has been droppped");
        MESHES_TO_DELETE.lock().unwrap().push(self.0);
    }
}
