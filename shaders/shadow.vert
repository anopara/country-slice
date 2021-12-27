#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec2 Vertex_UV;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 v_Uv;
out vec3 v_pos_ws;

uniform sampler2D terrain_texture;

float sample_terrain_texture_ws(vec2 pos_ws) {
    vec2 texture_uv = (pos_ws / 20.0 + 0.5);
    return texture(terrain_texture, texture_uv).x;
}

void main() {

    vec3 pos_ws = (model * vec4(Vertex_Position, 1.0)).xyz;
    pos_ws.y = sample_terrain_texture_ws(pos_ws.xz) + 0.02;

    gl_Position = projection * view * vec4(pos_ws, 1.0);
    v_Uv = Vertex_UV;
    v_pos_ws = pos_ws;
}