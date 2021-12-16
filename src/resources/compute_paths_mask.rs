use crate::{
    asset_libraries::{shader_library::AssetShaderLibrary, Handle},
    render::{shader::ShaderProgram, shaderwatch::ShaderWatch},
};

pub struct ComputePathsMask {
    pub compute_program: Handle<ShaderProgram>,
    pub texture: u32,
    pub texture_dims: (i32, i32),
}

impl ComputePathsMask {
    pub fn init(shaderwatch: &mut ShaderWatch, assets_library: &mut AssetShaderLibrary) -> Self {
        unsafe {
            let texture_dims = (512, 512);
            // Create texture
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F as i32,
                texture_dims.0,
                texture_dims.1,
                0,
                gl::RGBA,
                gl::FLOAT,
                std::ptr::null(),
            );
            // create shader program
            let shader_program = ShaderProgram::new_compute("shaders/compute_test.comp").unwrap();

            shaderwatch.watch(&shader_program);
            let handle = assets_library.add(shader_program.into());

            ComputePathsMask {
                compute_program: handle,
                texture,
                texture_dims,
            }
        }
    }
}
