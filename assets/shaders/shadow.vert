#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

layout(location = 1) out vec2 v_Uv;

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_Uv = Vertex_Uv;
}