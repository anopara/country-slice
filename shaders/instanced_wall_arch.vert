#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; 
layout (location = 2) in vec3 Vertex_Normal; 

out flat int instance_id;

out vec3 vertex_color; 
out vec3 vertex_normal_ws;
out vec3 vertex_position_ws;
out vec3 curve_position_ws;

uniform float wall_length;

layout (std430, binding=2) buffer transforms_buffer { 
    mat4 transforms[];
};

//uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

uniform sampler2D terrain_texture;

float sample_terrain_texture_ws(vec2 pos_ws) {
    vec2 texture_uv = (pos_ws / 20.0 + 0.5);
    return texture(terrain_texture, texture_uv).x;
}

void main()
{   
    mat4 instance_transform = transforms[gl_InstanceID];
    vec4 vertex_ws = instance_transform * vec4(Vertex_Position, 1.0);

    // ---------------------- TERRAIN

    //vertex_ws.y += sample_terrain_texture_ws(vertex_ws.xz);

    const float WALL_HEIGHT = 1.4;
    float height_u = vertex_ws.y / WALL_HEIGHT * 0.7;

    vec4 terrain_p = vertex_ws;
    terrain_p.y += sample_terrain_texture_ws(vertex_ws.xz);

    vertex_ws = mix(terrain_p, vertex_ws, height_u);


    // ----------------------------------

    gl_Position = projection * view * vertex_ws;
    instance_id = 0; //used for color, so put 0 here for now...
    vertex_color = vertex_color;
    vertex_position_ws = vertex_ws.xyz;
    vertex_normal_ws = (instance_transform * vec4(Vertex_Normal, 0.0)).xyz;

    curve_position_ws =vec3(0.0, 0.0, 0.0); // this is only used for discarding fragments, which is irrelevant for the arch
    
} 

