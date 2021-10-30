#version 450
layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {

    vec3 Pos = Vertex_Position + vec3(5.0, 0.0, 0.0);

    gl_Position = ViewProj * Model * vec4(Pos, 1.0);
}