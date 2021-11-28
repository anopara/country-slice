use gl::types::*;
use std::collections::HashMap;
use std::mem;
use std::os::raw::c_void;

pub struct Mesh {
    pub attributes: HashMap<String, VertexAttributeValues>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub const ATTRIBUTE_POSITION: &'static str = "Vertex_Position";
    pub const ATTRIBUTE_COLOR: &'static str = "Vertex_Color";
    pub const ATTRIBUTE_UV: &'static str = "Vertex_UV";
    pub const ATTRIBUTE_NORMAL: &'static str = "Vertex_Normal";

    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            indices: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.indices.is_empty()
    }

    pub fn set_attribute(&mut self, name: &str, values: impl Into<VertexAttributeValues>) {
        let name = String::from(name);
        if let Some(existing_entry) = self.attributes.get_mut(&name) {
            *existing_entry = values.into();
        } else {
            self.attributes.insert(name, values.into());
        }
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
    }
}

pub enum VertexAttributeValues {
    Sint32(Vec<i32>),
    Float32(Vec<f32>),
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
}

macro_rules! impl_from {
    ($type:ty, $variant:ident) => {
        impl From<$type> for VertexAttributeValues {
            fn from(v: $type) -> Self {
                Self::$variant(v)
            }
        }
    };
}

impl_from! {Vec<i32>, Sint32}
impl_from! {Vec<f32>, Float32}
impl_from! {Vec<[f32; 2]>, Float32x2}
impl_from! {Vec<[f32; 3]>, Float32x3}
impl_from! {Vec<[f32; 4]>, Float32x4}

impl VertexAttributeValues {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            // float
            VertexAttributeValues::Float32(values) => values.len() * mem::size_of::<GLfloat>(),
            VertexAttributeValues::Float32x2(values) => {
                values[0].len() * values.len() * mem::size_of::<GLfloat>()
            }
            VertexAttributeValues::Float32x3(values) => {
                values[0].len() * values.len() * mem::size_of::<GLfloat>()
            }
            VertexAttributeValues::Float32x4(values) => {
                values[0].len() * values.len() * mem::size_of::<GLfloat>()
            }
            // int
            VertexAttributeValues::Sint32(values) => values.len() * mem::size_of::<GLint>(),
        }
    }

    pub fn as_ptr(&self) -> *const c_void {
        match self {
            // float
            Self::Float32(values) => values.as_ptr() as *const c_void,
            Self::Float32x2(values) => values.as_ptr() as *const c_void,
            Self::Float32x3(values) => values.as_ptr() as *const c_void,
            Self::Float32x4(values) => values.as_ptr() as *const c_void,
            // int
            Self::Sint32(values) => values.as_ptr() as *const c_void,
        }
    }

    pub fn size(&self) -> i32 {
        match self {
            Self::Float32(_) | Self::Sint32(_) => 1,
            Self::Float32x2(values) => values[0].len() as i32,
            Self::Float32x3(values) => values[0].len() as i32,
            Self::Float32x4(values) => values[0].len() as i32,
        }
    }

    pub fn gl_type(&self) -> GLenum {
        match self {
            Self::Float32(_) | Self::Float32x2(_) | Self::Float32x3(_) | Self::Float32x4(_) => {
                gl::FLOAT
            }
            Self::Sint32(_) => gl::INT,
        }
    }

    pub fn stride(&self) -> GLsizei {
        match self {
            Self::Float32(_) | Self::Float32x2(_) | Self::Float32x3(_) | Self::Float32x4(_) => {
                self.size() * mem::size_of::<GLfloat>() as GLsizei
            }
            Self::Sint32(_) => self.size() * mem::size_of::<GLint>() as GLsizei,
        }
    }
}
