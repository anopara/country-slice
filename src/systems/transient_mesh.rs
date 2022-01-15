use bevy_ecs::prelude::ResMut;

use crate::{
    asset_libraries::{mesh_library::AssetMeshLibrary, vao_library::AssetVAOLibrary},
    components::MESHES_TO_DELETE,
};

pub fn delete_dropped_transient_meshes(
    mut assets_mesh: ResMut<AssetMeshLibrary>,
    mut assets_vao: ResMut<AssetVAOLibrary>,
) {
    for handle in MESHES_TO_DELETE.lock().unwrap().drain(..) {
        log::debug!("Deleting a transient mesh");

        // remove mesh from library
        assets_mesh.remove(handle);

        // remove mesh from VAO library & memory
        assets_vao.remove(handle);
    }
}
