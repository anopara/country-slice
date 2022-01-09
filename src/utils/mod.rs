pub mod load_gltf;
pub mod load_json;

pub mod custom_macro {

    macro_rules! log_if_error {
        ($expression: expr) => {
            $expression.unwrap_or_else(|err| log::error!("{}", err))
        };
    }

    pub(crate) use log_if_error;
}

use std::process::Command;

use bevy_ecs::{archetype::Archetypes, component::ComponentId, prelude::Entity};

pub fn validate_shaders(folder: &str) {
    // check if validator exists
    if !std::path::Path::new("glslangValidator.exe").exists() {
        log::error!("glslangValidator.exe not found");
        return;
    }

    if let Ok(paths) = std::fs::read_dir(folder) {
        for path in paths {
            let p = path.unwrap().path();
            let p = p.to_str().unwrap();

            if let Ok(output) = Command::new("glslangValidator").arg(p).status() {
                if !output.success() {
                    log::error!(
                        "Shader has failed validation. See the error above. status: {}",
                        output
                    );
                }
            } else {
                log::error!("failed to execute process");
                return;
            }
        }
        log::info!("Shader validation complete");
    } else {
        log::error!("The given path doesn't exist {}", folder);
    }
}

#[allow(dead_code)]
// from: https://github.com/bevyengine/bevy/discussions/3332
pub fn get_components_for_entity<'a>(
    entity: &Entity,
    archetypes: &'a Archetypes,
) -> Option<impl Iterator<Item = ComponentId> + 'a> {
    for archetype in archetypes.iter() {
        if archetype.entities().contains(entity) {
            return Some(archetype.components());
        }
    }
    None
}

/*
pub fn iter_mesh_ws_vertex_positions<'a>(
    handle: Handle<Mesh>,
    transform: &'a Transform,
    assets_mesh: &'a mut ResMut<AssetMeshLibrary>,
) -> impl Iterator<Item = Vec3> + 'a {
    let mesh = assets_mesh.get_mut(handle).unwrap();
    let mesh_ws_pos = mesh.attributes.get(Mesh::ATTRIBUTE_POSITION).unwrap();

    if let crate::render::mesh::VertexAttributeValues::Float32x3(positions) = mesh_ws_pos {
        positions.iter().map(move |p| {
            transform
                .compute_matrix()
                .transform_point3(Vec3::from_slice(p))
        })
    } else {
        panic!()
    }
}
 */
