use std::{ffi::CString, marker::PhantomData, os::raw::c_char, ptr};

use gl::types::GLsizeiptr;

use super::shader::ShaderProgram;

pub struct GLShaderStorageBuffer<T> {
    id: u32,
    buffer_size: usize,

    // store how many instances this buffer is for
    pub instance_num: usize,
    //
    pub binding_point: u32,
    _marker: PhantomData<T>,
}

// Storage buffer stores the information about instance transforms

impl<T: Copy> GLShaderStorageBuffer<T> {
    pub fn new(data: &Vec<T>, buffer_size: usize, binding_point: u32) -> Self {
        Self {
            id: unsafe { create_storage_buffer::<T>(buffer_size) },
            buffer_size,
            instance_num: data.len(),
            binding_point,
            _marker: PhantomData,
        }
    }

    pub fn update(&mut self, data: &[T]) {
        unsafe {
            assert!(data.len() <= self.buffer_size);

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.id);
            let ptr = gl::MapBuffer(gl::SHADER_STORAGE_BUFFER, gl::WRITE_ONLY);

            assert!(!ptr.is_null());

            let dst =
                std::slice::from_raw_parts_mut(ptr as *mut T, self.buffer_size.min(data.len()));
            dst.copy_from_slice(data);
            gl::UnmapBuffer(gl::SHADER_STORAGE_BUFFER);

            self.instance_num = data.len();
        }
    }

    pub fn bind(&self, shader_program: &ShaderProgram, name: &str) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            let block_index = gl::GetProgramResourceIndex(
                shader_program.id(),
                gl::SHADER_STORAGE_BLOCK,
                c_str.as_ptr() as *const c_char,
            );
            gl::ShaderStorageBlockBinding(shader_program.id(), block_index, self.binding_point);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, self.binding_point, self.id);
        }
    }

    pub fn gl_id(&self) -> u32 {
        self.id
    }
}

pub unsafe fn create_storage_buffer<T>(size: usize) -> u32 {
    let mut ssbo = 0; // shader storage buffer object
    gl::GenBuffers(1, &mut ssbo);

    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
    gl::BufferData(
        gl::SHADER_STORAGE_BUFFER,
        (std::mem::size_of::<T>() * size) as GLsizeiptr,
        ptr::null(),
        gl::STATIC_DRAW,
    );
    // Unbind
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

    log::debug!("Created a storage buffer {}", ssbo);

    ssbo
}
