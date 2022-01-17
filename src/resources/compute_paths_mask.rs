use crate::{
    asset_libraries::{shader_library::AssetShaderLibrary, Handle},
    render::{shader::ShaderProgram, shaderwatch::ShaderWatch, texture::GlTextureRGBAf32},
};

pub struct ComputePathMaskBlur {
    pub compute_program: Handle<ShaderProgram>,
    pub texture: u32,
    pub texture_dims: (i32, i32),
}

pub struct ComputePathMask {
    pub compute_program: Handle<ShaderProgram>,
    pub texture: GlTextureRGBAf32,
}

impl ComputePathMask {
    pub fn init(shaderwatch: &mut ShaderWatch, assets_library: &mut AssetShaderLibrary) -> Self {
        unsafe {
            let texture = GlTextureRGBAf32::new((512, 512), None);
            // create shader program
            let shader_program =
                ShaderProgram::new_compute("shaders/compute_path_mask.comp").unwrap();

            shaderwatch.watch(&shader_program);
            let handle = assets_library.add(shader_program.into());

            ComputePathMask {
                compute_program: handle,
                texture,
            }
        }
    }
}
