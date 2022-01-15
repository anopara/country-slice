use bevy_ecs::prelude::*;

use crate::{
    asset_libraries::shader_library::AssetShaderLibrary, render::shaderwatch::ShaderWatch,
};

pub fn shaderwatch(
    shaderwatch: ResMut<ShaderWatch>,
    mut assets_shader: ResMut<AssetShaderLibrary>,
) {
    let mut changed_shaders = shaderwatch.event_shader_changed.lock().unwrap();

    if !changed_shaders.is_empty() {
        log::info!("Shaderwatch: detected changes: {:?}", changed_shaders);

        for (_, shader) in &mut assets_shader.assets {
            // if any of the source code has changed, the shader needs recompilation
            if shader
                .src_paths()
                .drain(..)
                .find(|p| changed_shaders.contains(*p))
                .is_some()
            {
                if let Err(error) = shader.recompile() {
                    log::error!(
                        "Failed to recompile shader: {}; thread: {:?}",
                        error,
                        std::thread::current()
                    );
                }
            }
        }

        changed_shaders.clear();
    }
}
