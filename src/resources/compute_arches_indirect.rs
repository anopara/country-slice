use gl::types::GLsizeiptr;
use glam::Vec3;

use crate::{
    asset_libraries::{shader_library::AssetShaderLibrary, Handle},
    render::{
        shader::{GlUniform, ShaderProgram},
        shaderwatch::ShaderWatch,
        ssbo::GLShaderStorageBuffer,
    },
    resources::compute_path_mask::PATH_MASK_WS_DIMS,
    utils::custom_macro::log_if_error,
};

const COMMAND_BUFFER_SIZE: usize = 1000;

pub struct ComputeArchesIndirect {
    pub compute_program: Handle<ShaderProgram>,
    pub draw_indirect_cmd_buffer: u32, //draw indirect
    pub cmd_buffer_binding_point: u32,
    //
    pub transforms_buffer: GLShaderStorageBuffer<glam::Mat4>,
    //
    pub curves_buffer: GLShaderStorageBuffer<CurveDataSSBO>,
}

impl ComputeArchesIndirect {
    pub fn init(shaderwatch: &mut ShaderWatch, assets_library: &mut AssetShaderLibrary) -> Self {
        unsafe {
            // create shader program
            let shader_program =
                ShaderProgram::new_compute("shaders/arch_layout_bricks.comp").unwrap(); //indirect_draw_test
            shaderwatch.watch(&shader_program);
            let handle = assets_library.add(shader_program.into());

            // Setup GL_DRAW_INDIRECT_BUFFER for indirect drawing (basically a command buffer)
            let mut ibo = 0;
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, ibo);
            // Unlike `gl::BufferData`, `glBufferStorage` don't allow you to de-allocate it
            // that means the buffer is persistented in the memory, and you don't need to unmap until you really don't need it
            // (c) https://stackoverflow.com/questions/27810542/what-is-the-difference-between-glbufferstorage-and-glbufferdata
            gl::BufferStorage(
                gl::DRAW_INDIRECT_BUFFER,
                (std::mem::size_of::<DrawElementsIndirectCommand>() * COMMAND_BUFFER_SIZE)
                    as GLsizeiptr,
                std::ptr::null(),
                gl::MAP_READ_BIT | gl::MAP_WRITE_BIT,
            );

            ComputeArchesIndirect {
                compute_program: handle,
                draw_indirect_cmd_buffer: ibo,
                cmd_buffer_binding_point: 0,
                transforms_buffer: GLShaderStorageBuffer::<glam::Mat4>::new(&vec![], 10000, 2),
                curves_buffer: GLShaderStorageBuffer::<CurveDataSSBO>::new(&vec![], 1000, 3),
            }
        }
    }

    pub fn bind(
        &self,
        assets_shader: &AssetShaderLibrary,
        segments_buffer: &GLShaderStorageBuffer<super::ArchSegmentDataSSBO>,
        path_mask: u32,
        img_unit: u32,
    ) {
        unsafe {
            // bind compute shader
            let shader = assets_shader.get(self.compute_program).unwrap();
            gl::UseProgram(shader.id());

            // bind command buffer
            //from: https://lingtorp.com/2018/12/05/OpenGL-SSBO-indirect-drawing.html

            let c_str = std::ffi::CString::new("draw_commands").unwrap();
            let block_index = gl::GetProgramResourceIndex(
                shader.id(),
                gl::SHADER_STORAGE_BLOCK,
                c_str.as_ptr() as *const std::os::raw::c_char,
            );
            gl::ShaderStorageBlockBinding(shader.id(), block_index, self.cmd_buffer_binding_point);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.draw_indirect_cmd_buffer);
            gl::BindBufferBase(
                gl::SHADER_STORAGE_BUFFER,
                self.cmd_buffer_binding_point,
                self.draw_indirect_cmd_buffer,
            );

            // bind transforms buffer
            self.transforms_buffer.bind(&shader, "transforms_buffer");

            // bind road mask
            log_if_error!(shader.set_gl_uniform("path_mask", GlUniform::Int(img_unit as i32),));
            log_if_error!(
                shader.set_gl_uniform("path_mask_ws_dims", GlUniform::Vec2(PATH_MASK_WS_DIMS))
            );
            // bind texture
            gl::BindImageTexture(
                img_unit,
                path_mask,
                0,
                gl::FALSE,
                0,
                gl::READ_ONLY,
                gl::RGBA32F,
            );

            // bind curve ssbo
            //self.curves_buffer.bind(shader, "curves_buffer");

            // bind segments buffer
            segments_buffer.bind(&shader, "segments_buffer");
        }
    }

    pub fn reset_draw_command_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.draw_indirect_cmd_buffer);
            let ptr = gl::MapBuffer(gl::DRAW_INDIRECT_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut DrawElementsIndirectCommand, 1);
            dst.copy_from_slice(&[DrawElementsIndirectCommand {
                _count: 312, // number of vertices of brick.glb
                _instance_count: 0,
                _first_index: 0,
                _base_vertex: 0,
                _base_instance: 0,
            }]);
            gl::UnmapBuffer(gl::DRAW_INDIRECT_BUFFER);
        }
    }

    pub fn reset_transform_buffer(&self) {
        unsafe {
            let data = &[glam::Mat4::IDENTITY; 10000];
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.transforms_buffer.gl_id());
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst = std::slice::from_raw_parts_mut(ptr as *mut glam::Mat4, data.len());
            dst.copy_from_slice(data);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct DrawElementsIndirectCommand {
    pub _count: u32,
    pub _instance_count: u32,
    pub _first_index: u32,
    pub _base_vertex: u32,
    pub _base_instance: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CurveDataSSBO {
    pub points_count: u32,
    pub pad0: u32,
    pub pad1: u32,
    pub pad2: u32,
    pub positions: [[f32; 4]; 1000], //buffer
}

impl CurveDataSSBO {
    pub fn from(curve: &crate::geometry::curve::Curve) -> Self {
        let points_count = curve.points.len() as u32;
        let mut positions = [[0.0; 4]; 1000];

        positions.iter_mut().enumerate().for_each(|(i, p)| {
            *p = curve
                .points
                .get(i)
                .unwrap_or(&Vec3::ZERO)
                .extend(1.0)
                .to_array()
        });

        Self {
            points_count,
            pad0: 0,
            pad1: 0,
            pad2: 0,
            positions,
        }
    }

    pub fn empty() -> Self {
        CurveDataSSBO {
            points_count: 0,
            pad0: 0,
            pad1: 0,
            pad2: 0,
            positions: [[0.0; 4]; 1000],
        }
    }
}
