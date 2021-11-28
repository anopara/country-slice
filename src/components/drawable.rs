use bevy_ecs::prelude::Bundle;
use gl::types::GLenum;

use crate::{
    asset_libraries::Handle,
    render::{mesh::Mesh, shader::ShaderProgram},
};

use super::transform::Transform;

#[derive(Bundle)]
pub struct DrawableMeshBundle {
    pub mesh: Handle<Mesh>,
    pub shader: Handle<ShaderProgram>,
    pub transform: Transform,
}

pub struct GLDrawMode(pub GLenum);

pub struct TransparencyPass;
