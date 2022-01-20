use glam::Vec3;
use std::ffi::CString;

use crate::{
    asset_libraries::shader_library::AssetShaderLibrary,
    render::{shader::GlUniform, shaderwatch::ShaderWatch},
    systems::mode_manager::{BrushMode, EraseLayer},
    utils::custom_macro::log_if_error,
};

use super::compute_textures::ComputeTexture;

// path mask is a texture/plane centered on 0.0 with bounds from -10 to 10
pub const PATH_MASK_WS_DIMS: [f32; 2] = [20.0, 20.0];

pub struct PathMaskComputePass {
    pub path_mask: ComputeTexture,         // Draw pass
    pub path_mask_blurred: ComputeTexture, // Blur pass
}

impl PathMaskComputePass {
    pub fn init(shaderwatch: &mut ShaderWatch, assets_library: &mut AssetShaderLibrary) -> Self {
        let path_mask = ComputeTexture::init(
            "shaders/compute_path_mask.comp",
            shaderwatch,
            assets_library,
        );
        let path_mask_blurred =
            ComputeTexture::init("shaders/blur.comp", shaderwatch, assets_library);

        Self {
            path_mask,
            path_mask_blurred,
        }
    }

    pub fn bind_draw_pass(
        &self,
        mode: &BrushMode,
        assets_shader: &AssetShaderLibrary,
        mouse_ws: &Vec3,
        img_unit: u32,
    ) {
        unsafe {
            if (matches!(mode, BrushMode::Path)
                || matches!(mode, BrushMode::Eraser(EraseLayer::All)))
            {
                let shader = assets_shader.get(self.path_mask.compute_program).unwrap();

                gl::UseProgram(shader.id());

                match mode {
                    BrushMode::Wall => panic!(),
                    BrushMode::Path => {
                        log_if_error!(shader.set_gl_uniform("is_additive", GlUniform::Bool(true)))
                    }
                    BrushMode::Eraser(..) => {
                        log_if_error!(shader.set_gl_uniform("is_additive", GlUniform::Bool(false)))
                    }
                }

                // connect shader's uniform variable to our texture
                // instead of name can specify in shader the binding, for ex "layout(rgba32f, binding = 0)"
                let uniform_name = CString::new("img_output").unwrap();
                let tex_location =
                    gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
                gl::Uniform1i(tex_location, img_unit as i32);

                // bind texture
                gl::BindImageTexture(
                    img_unit,
                    self.path_mask.texture.id,
                    0,
                    gl::FALSE,
                    0,
                    gl::READ_WRITE,
                    gl::RGBA32F,
                );

                log_if_error!(
                    shader.set_gl_uniform("Mouse_Position", GlUniform::Vec3(mouse_ws.to_array()))
                );
                log_if_error!(shader.set_gl_uniform(
                    "path_mask_ws_dims",
                    GlUniform::Vec2(crate::resources::compute_path_mask::PATH_MASK_WS_DIMS)
                ));
            }
        }
    }

    pub fn bind_blur_pass(&self, assets_shader: &AssetShaderLibrary, mut img_unit: u32) {
        unsafe {
            let shader = assets_shader
                .get(self.path_mask_blurred.compute_program)
                .unwrap();
            gl::UseProgram(shader.id());

            let uniform_name = CString::new("img_in").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1i(tex_location, img_unit as i32);

            // bind texture
            gl::BindImageTexture(
                img_unit,
                self.path_mask.texture.id,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );
            img_unit += 1;

            let uniform_name = CString::new("img_out").unwrap();
            let tex_location =
                gl::GetUniformLocation(shader.id(), uniform_name.as_ptr() as *const i8);
            gl::Uniform1i(tex_location, img_unit as i32);

            // bind texture
            gl::BindImageTexture(
                img_unit,
                self.path_mask_blurred.texture.id,
                0,
                gl::FALSE,
                0,
                gl::READ_WRITE,
                gl::RGBA32F,
            );
        }
    }
}
