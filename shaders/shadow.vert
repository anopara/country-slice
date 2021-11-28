#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_UV;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 v_Uv;

void main() {
    gl_Position = projection * view * model * vec4(Vertex_Position, 1.0);
    v_Uv = Vertex_UV;
}