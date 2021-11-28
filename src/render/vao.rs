use super::{mesh::Mesh, shader::ShaderProgramId};
use gl::types::*;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;

pub struct GLVertexArray(pub u32);

pub struct GLVertexBuffer(pub u32);

pub struct VAO {
    pub id: GLVertexArray,
    pub indices_count: usize,
    pub vertex_buffer_ids: Vec<GLVertexBuffer>,
}

impl VAO {
    pub fn id(&self) -> u32 {
        self.id.0
    }

    pub fn new(mesh: &Mesh, shader_program_id: &ShaderProgramId) -> Self {
        let (id, vertex_buffer_ids) = build_vao(mesh, shader_program_id);
        Self {
            id,
            vertex_buffer_ids,
            indices_count: mesh.indices.len(),
        }
    }

    pub fn rebuild(&mut self, mesh: &Mesh, shader_program_id: &ShaderProgramId) {
        self.clear();
        let (id, vertex_buffer_ids) = build_vao(mesh, shader_program_id);
        self.id = id;
        self.vertex_buffer_ids = vertex_buffer_ids;
        self.indices_count = mesh.indices.len();
    }

    fn clear(&mut self) {
        unsafe {
            for vbo in &mut self.vertex_buffer_ids {
                //println!("----------------- VBO ID {} is deleted", vbo.0);
                gl::DeleteBuffers(1, &mut vbo.0);
            }
            gl::DeleteVertexArrays(1, &mut self.id.0);
        }
    }
}

fn build_vao(
    mesh: &Mesh,
    shader_program_id: &ShaderProgramId,
) -> (GLVertexArray, Vec<GLVertexBuffer>) {
    let mut vao = 0; //vertex attribute object
    let mut vbo: u32 = 0; // vertex buffer object
    let mut ebo = 0; // element (index) buffer

    let mut vbos = Vec::new();

    unsafe {
        // 1. bind Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // 2. copy our index array in a element buffer for OpenGL to use
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (mesh.indices.len() * mem::size_of::<GLint>()) as GLsizeiptr,
            &mesh.indices[0] as *const u32 as *const c_void,
            gl::STATIC_DRAW,
        );

        for (name, attribute) in mesh.attributes.iter() {
            // 3. for every attribute, generate a vbo
            gl::GenBuffers(1, &mut vbo);
            //println!("----------------- VBO ID is {}", vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                // the size of the data (in bytes)
                attribute.size_in_bytes() as GLsizeiptr,
                // pass the data
                attribute.as_ptr(),
                // GL_STREAM_DRAW: the data is set only once and used by the GPU at most a few times.
                // GL_STATIC_DRAW: the data is set only once and used many times.
                // GL_DYNAMIC_DRAW: the data is changed a lot and used many times.
                // with data that is likely to change frequently, a usage type of GL_DYNAMIC_DRAW ensures the graphics card will place the data in memory that allows for faster writes.
                gl::STATIC_DRAW,
            );
            // 4. then set our vertex attributes pointers
            // tell OpenGL how to link the vertex data in VBO to the vertex shader's vertex attributes
            /*
            Each vertex attribute takes its data from memory managed by a VBO and
            which VBO it takes its data from is determined by the VBO currently
            bound to GL_ARRAY_BUFFER when calling glVertexAttribPointer. (you can have multiple VBOs).
            Since the previously defined VBO is still bound before calling glVertexAttribPointer
            vertex attribute 0 is now associated with its vertex data.
            */

            let name_str = CString::new(name.as_str()).unwrap();
            let layout = gl::GetAttribLocation(shader_program_id.0, name_str.as_ptr());
            gl::VertexAttribPointer(
                layout as u32,       // This sets the location of the vertex attribute to (layout = 0)
                attribute.size(), // specifies the size of the vertex attribute. The position attribute is a vec3 so it is composed of 3 values
                attribute.gl_type(), //specifies the type of the data
                gl::FALSE,        // specifies if we want the data to be normalized.
                attribute.stride(), // the stride tells us the space between consecutive vertex attributes
                ptr::null(), // the offset of where the position data begins in the buffer. Since the position data is at the start of the data array this value is just 0.
            );
            // enable the vertex attribute giving the vertex attribute location as its argument
            gl::EnableVertexAttribArray(layout as u32);
            vbos.push(GLVertexBuffer(vbo));
        }
    }

    (GLVertexArray(vao), vbos)
}
