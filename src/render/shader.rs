use gl::types::*;

use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::ptr;
use std::str;

#[allow(dead_code)]
pub enum GlUniform {
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4([f32; 16]),
}

#[derive(Clone, PartialEq, Eq)]
pub enum ShaderType {
    Graphics {
        vertex_shader_path: String,
        fragment_shader_path: String,
    },
    Compute {
        compute_shader_path: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderProgramId(pub u32);

#[derive(Clone, PartialEq, Eq)]
pub struct ShaderProgram {
    pub id: ShaderProgramId,
    pub ty: ShaderType,
}

impl ShaderProgram {
    pub fn new_compute(compute_shader_path: &str) -> Result<Self, String> {
        let shader_source = read_file_to_string(compute_shader_path);
        let shader = compile_shader(gl::COMPUTE_SHADER, &shader_source)?;
        let shader_program_id = link_shader_program(&[shader])?;
        Ok(Self {
            id: shader_program_id,
            ty: ShaderType::Compute {
                compute_shader_path: String::from(compute_shader_path),
            },
        })
    }

    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<Self, String> {
        // 1. retrieve the vertex/fragment source code from filesystem
        let vertex_src = read_file_to_string(vertex_shader_path);
        let frag_src = read_file_to_string(fragment_shader_path);

        // Compile shaders
        let vertex_shader = compile_shader(gl::VERTEX_SHADER, vertex_src.as_str())?;
        let frag_shader = compile_shader(gl::FRAGMENT_SHADER, frag_src.as_str())?;
        // Create, link & compile Shader Program object

        let shader_program = link_shader_program(&[vertex_shader, frag_shader])?;

        Ok(Self {
            id: shader_program,
            ty: ShaderType::Graphics {
                vertex_shader_path: String::from(vertex_shader_path),
                fragment_shader_path: String::from(fragment_shader_path),
            },
        })
    }

    pub fn recompile(&mut self) -> Result<(), String> {
        let old_shader_id = self.id.0;

        match &self.ty {
            ShaderType::Graphics {
                vertex_shader_path,
                fragment_shader_path,
            } => {
                *self =
                    ShaderProgram::new(vertex_shader_path.as_str(), fragment_shader_path.as_str())?
            }
            ShaderType::Compute {
                compute_shader_path,
            } => *self = ShaderProgram::new_compute(compute_shader_path.as_str())?,
        }

        unsafe {
            gl::DeleteProgram(old_shader_id);
        }
        Ok(())
    }

    pub fn src_paths(&self) -> Vec<&String> {
        match &self.ty {
            ShaderType::Graphics {
                vertex_shader_path,
                fragment_shader_path,
            } => vec![vertex_shader_path, fragment_shader_path],
            ShaderType::Compute {
                compute_shader_path,
            } => vec![compute_shader_path],
        }
    }

    pub fn id(&self) -> u32 {
        self.id.0
    }

    /// activate the shader
    /// ------------------------------------------------------------------------
    pub unsafe fn gl_use_program(&self) {
        gl::UseProgram(self.id())
    }

    /// utility uniform function
    /// ------------------------------------------------------------------------
    #[inline(always)]
    pub unsafe fn set_gl_uniform(&self, name: &str, value: GlUniform) -> Result<(), String> {
        let c_str = CString::new(name.as_bytes())
            .expect(&format!("Couldn't create a CString from {}", name));
        let location = gl::GetUniformLocation(self.id(), c_str.as_ptr());
        if location == -1 {
            return Err(format!("Couldn't Get Uniform Location {}", name));
        } else {
            match value {
                GlUniform::Bool(value) => gl::Uniform1i(location, value as i32),
                GlUniform::Int(value) => gl::Uniform1i(location, value),
                GlUniform::Float(value) => gl::Uniform1f(location, value),
                GlUniform::Vec3(v) => gl::Uniform3f(location, v[0], v[1], v[2]),
                GlUniform::Vec4(v) => gl::Uniform4f(location, v[0], v[1], v[2], v[3]),
                GlUniform::Mat4(matrix) => {
                    gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr())
                }
            };
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Shader {
    pub id: GLuint,
}

pub fn read_file_to_string(path: &str) -> String {
    let mut file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));

    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect(format!("Failed to read {}", path).as_str());

    contents
}

pub fn compile_shader(shader_type: gl::types::GLenum, source: &str) -> Result<Shader, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str_vert = CString::new(source.as_bytes()).unwrap();
        // The glShaderSource function takes the shader object to compile to as its first argument.
        // The second argument specifies how many strings we're passing as source code, which is only one.
        // The third parameter is the actual source code of the vertex shader and we can leave the 4th parameter to NULL.
        gl::ShaderSource(shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // check for shader compile errors
        let mut success = gl::FALSE as GLint;
        let mut info_log = vec![0; 513];
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );

            let info_log = CStr::from_ptr(info_log.as_ptr()).to_string_lossy();

            let error = format!(
                "ERROR::SHADER::COMPILATION_FAILED\n{}",
                info_log //, source
            );
            log::error!("{}", info_log);
            return Err(error);
        }

        Ok(Shader { id: shader })
    }
}

pub fn link_shader_program(shaders: &[Shader]) -> Result<ShaderProgramId, String> {
    unsafe {
        // creates a program and returns the ID reference to the newly created program object.
        let shader_program = gl::CreateProgram();
        // attach the previously compiled shaders to the program object and then link them
        for shader in shaders {
            gl::AttachShader(shader_program, shader.id);
        }
        gl::LinkProgram(shader_program);
        // check for linking errors
        let mut success = gl::FALSE as GLint;
        let mut info_log = vec![0; 513];
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );

            let info_log = CStr::from_ptr(info_log.as_ptr()).to_string_lossy();

            let error = format!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", info_log);
            return Err(error);
        }
        // activate Shader Program
        gl::UseProgram(shader_program);
        // delete the shader objects once we've linked them into the program object; we no longer need them anymore
        for shader in shaders {
            gl::DeleteShader(shader.id);
        }

        Ok(ShaderProgramId(shader_program))
    }
}
