#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; // the color variable has attribute position 1
  
out vec3 ourColor; // output a color to the fragment shader

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform sampler2D terrain_texture;
float sample_terrain_texture_ws(vec2 pos_ws) {
    vec2 texture_uv = (pos_ws / 20.0 + 0.5);
    return texture(terrain_texture, texture_uv).x;
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main()
{   

    float h = sample_terrain_texture_ws(Vertex_Position.xz) + 0.5;
    h = fit01(h, 0.0, 1.0);
    h = clamp(h, 0.15, 1.0);

    // fall off

    float f = length(Vertex_Position / 10.0);
    f = pow(f, 3.0);
    //f = clamp(f, 0.0, 1.0);
    f = smoothstep(0.0, 1.0, f);

    vec3 pos_ws = Vertex_Position;
    pos_ws.y = sample_terrain_texture_ws(Vertex_Position.xz);

    gl_Position = projection * view * model * vec4(pos_ws, 1.0);
    ourColor = vec3(f);
    ourColor = mix(Vertex_Color * h, vec3(0.120741), f); // set ourColor to the input color we got from the vertex data //vec3(h, 0.0, Vertex_Position.y);//
} 