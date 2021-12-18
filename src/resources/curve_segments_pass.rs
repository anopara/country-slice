// TODO:
// break it down into 2 compute shaders
// 1. isolate_arch_segments
// 2. place_arch_bricks

use gl::types::GLsizeiptr;

use crate::{
    asset_libraries::{shader_library::AssetShaderLibrary, Handle},
    render::{self, shader::ShaderProgram, shaderwatch::ShaderWatch, ssbo::GLShaderStorageBuffer},
    utils::custom_macro::log_if_error,
};

use super::CurveDataSSBO;

const COMMAND_BUFFER_SIZE: usize = 1000;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ArchSegmentDataSSBO {
    pub start: [f32; 2], // x and z coordinate
    pub end: [f32; 2],
}

impl Default for ArchSegmentDataSSBO {
    fn default() -> Self {
        ArchSegmentDataSSBO {
            start: [0.0, -1.0],
            end: [0.0, -1.0],
        }
    }
}

pub struct CurveSegmentsComputePass {
    pub compute_program: Handle<ShaderProgram>,
    pub compute_indirect_cmd_buffer: u32, // compute indirect
    pub cmd_buffer_binding_point: u32,
    //
    pub curves_buffer: GLShaderStorageBuffer<CurveDataSSBO>, // read from
    pub segments_buffer: GLShaderStorageBuffer<ArchSegmentDataSSBO>, // write to
}

impl CurveSegmentsComputePass {
    pub fn init(shaderwatch: &mut ShaderWatch, assets_library: &mut AssetShaderLibrary) -> Self {
        unsafe {
            // create shader program
            let shader_program =
                ShaderProgram::new_compute("shaders/arch_curve_segments.comp").unwrap();
            shaderwatch.watch(&shader_program);
            let handle = assets_library.add(shader_program.into());

            // Setup GL_DISPATCH_INDIRECT_BUFFER for indirect dispatch of the next compute pass
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::DISPATCH_INDIRECT_BUFFER, id);
            gl::BufferStorage(
                gl::DISPATCH_INDIRECT_BUFFER,
                (std::mem::size_of::<DispatchIndirectCommand>() * COMMAND_BUFFER_SIZE)
                    as GLsizeiptr,
                std::ptr::null(),
                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
            );

            CurveSegmentsComputePass {
                compute_program: handle,
                compute_indirect_cmd_buffer: id,
                cmd_buffer_binding_point: 5,
                curves_buffer: GLShaderStorageBuffer::<CurveDataSSBO>::new(&vec![], 1000, 3),
                segments_buffer: GLShaderStorageBuffer::<ArchSegmentDataSSBO>::new(
                    &vec![],
                    1000,
                    4,
                ),
            }
        }
    }

    pub fn bind(
        &self,
        assets_shader: &AssetShaderLibrary,
        road_mask: u32,
        road_mask_img_unit: u32,
    ) {
        unsafe {
            // bind compute shader
            let shader = assets_shader.get(self.compute_program).unwrap();
            gl::UseProgram(shader.id());

            // bind command buffer
            //from: https://lingtorp.com/2018/12/05/OpenGL-SSBO-indirect-drawing.html

            let c_str = std::ffi::CString::new("dispatch_indirect").unwrap();
            let block_index = gl::GetProgramResourceIndex(
                shader.id(),
                gl::SHADER_STORAGE_BLOCK,
                c_str.as_ptr() as *const std::os::raw::c_char,
            );
            gl::ShaderStorageBlockBinding(shader.id(), block_index, self.cmd_buffer_binding_point);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.compute_indirect_cmd_buffer);
            gl::BindBufferBase(
                gl::SHADER_STORAGE_BUFFER,
                self.cmd_buffer_binding_point,
                self.compute_indirect_cmd_buffer,
            );

            // bind road mask
            log_if_error!(shader.set_gl_uniform(
                "road_mask",
                render::shader::GlUniform::Int(road_mask_img_unit as i32),
            ));
            // bind texture
            gl::BindImageTexture(
                road_mask_img_unit,
                road_mask,
                0,
                gl::FALSE,
                0,
                gl::READ_ONLY,
                gl::RGBA32F,
            );

            // bind curve ssbo
            self.curves_buffer.bind(shader, "curves_buffer");

            // bind segments buffer
            self.segments_buffer.bind(&shader, "output_segments_buffer");
        }
    }

    pub fn reset_cmd_buffer(&self) {
        unsafe {
            gl::BindBuffer(
                gl::DISPATCH_INDIRECT_BUFFER,
                self.compute_indirect_cmd_buffer,
            );
            let ptr = gl::MapBuffer(gl::DISPATCH_INDIRECT_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut DispatchIndirectCommand, 1);
            dst.copy_from_slice(&[DispatchIndirectCommand {
                _num_groups_x: 0,
                _num_groups_y: 1,
                _num_groups_z: 1,
            }]);
            gl::UnmapBuffer(gl::DISPATCH_INDIRECT_BUFFER);
        }
    }

    pub fn reset_segments_buffer(&self) {
        unsafe {
            let data = &[ArchSegmentDataSSBO::default(); 1000];
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.segments_buffer.gl_id());
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut ArchSegmentDataSSBO, data.len());
            dst.copy_from_slice(data);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DispatchIndirectCommand {
    pub _num_groups_x: u32,
    pub _num_groups_y: u32,
    pub _num_groups_z: u32,
}
