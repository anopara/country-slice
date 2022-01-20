use crate::{
    asset_libraries::{shader_library::AssetShaderLibrary, Handle},
    render::{shader::ShaderProgram, shaderwatch::ShaderWatch, texture::GlTextureRGBAf32},
};

pub struct ComputeTexture {
    pub compute_program: Handle<ShaderProgram>,
    pub texture: GlTextureRGBAf32,
}

impl ComputeTexture {
    pub fn init(
        compute_shader: &str,
        shaderwatch: &mut ShaderWatch,
        assets_library: &mut AssetShaderLibrary,
    ) -> Self {
        let texture = GlTextureRGBAf32::new((512, 512), None);
        let shader_program = ShaderProgram::new_compute(compute_shader).unwrap();

        shaderwatch.watch(&shader_program);
        let handle = assets_library.add(shader_program.into());

        ComputeTexture {
            compute_program: handle,
            texture,
        }
    }
}
