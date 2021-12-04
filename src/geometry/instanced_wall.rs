use gl::types::*;
use glam::Mat4;
use std::{ffi::CString, marker::PhantomData, os::raw::c_char, ptr};

use crate::render::shader::ShaderProgram;

use super::wall_constructor::Brick;

#[repr(C)]
pub struct InstancedWall {
    pub wall_length: f32,
    pub instance_buffer: GLShaderStorageBuffer<InstancedWallData>, //Vec<InstancedWallData>,
}

impl InstancedWall {
    fn instanced_wall_data(bricks: Vec<Brick>) -> Vec<InstancedWallData> {
        bricks
            .iter()
            .map(|b| {
                let min = b.pivot_uv - b.bounds_uv / 2.0;
                let max = b.pivot_uv + b.bounds_uv / 2.0;

                InstancedWallData {
                    transform: b.transform.compute_matrix(),
                    curve_uv_bbx_minmax: [min.x, min.y, max.x, max.y],
                }
            })
            .collect()
    }

    pub fn from(curve_length: f32, bricks: Vec<Brick>) -> Self {
        Self {
            wall_length: curve_length,
            instance_buffer: GLShaderStorageBuffer::<InstancedWallData>::new(
                &Self::instanced_wall_data(bricks),
            ),
        }
    }

    pub fn update(&mut self, curve_length: f32, bricks: Vec<Brick>) {
        self.wall_length = curve_length;
        self.instance_buffer
            .update(&Self::instanced_wall_data(bricks));
    }
}

//TODO: rename into BrickTransform ?
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct InstancedWallData {
    transform: Mat4,
    curve_uv_bbx_minmax: [f32; 4],
}

// ----------------------------------------- Shader stuff

const BUFFER_SIZE: usize = 10000;
const BINDING_POINT: u32 = 2;

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
    pub fn new(data: &Vec<T>) -> Self {
        Self {
            id: unsafe { create_storage_buffer::<T>(BUFFER_SIZE) }, //, data.as_ptr() as *const c_void) },
            buffer_size: BUFFER_SIZE,
            instance_num: data.len(),
            binding_point: BINDING_POINT,
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
}

pub unsafe fn create_storage_buffer<T>(size: usize) -> u32 {
    let mut ssbo = 0; // shader storage buffer object
    gl::GenBuffers(1, &mut ssbo);

    println!("Created a storage buffer {}", ssbo);

    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, ssbo);
    gl::BufferData(
        gl::SHADER_STORAGE_BUFFER,
        (std::mem::size_of::<T>() * size) as GLsizeiptr,
        ptr::null(),
        gl::STATIC_DRAW,
    );
    // Unbind
    gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);

    println!("Created SSBO");

    ssbo
}
