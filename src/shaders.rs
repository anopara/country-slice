use bevy::{reflect::TypeUuid, render::renderer::RenderResources};

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "93fb26fc-6c05-489b-9029-601edf703b6b"]
pub struct TimeUniform {
    pub value: f32,
}

pub const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    // Convert to clipspace
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_color = Vertex_Color;
}
"#;

pub const POSITION_TO_VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);

    // I know that brick is within [-0.5; 0.5] bounding box
    v_color = Vertex_Position + 0.5;
}
"#;

pub const FRAGMENT_SHADER_ANIMATED: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
layout(set = 2, binding = 0) uniform TimeUniform_value {
    float time;
};
void main() {
    o_Target = vec4(fract(v_color+time*0.5), 1.0);
}
"#;

pub const FRAGMENT_SHADER: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
void main() {
    o_Target = vec4(v_color, 1.0);
}
"#;
