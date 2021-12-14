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

void main()
{   
    mat4 instance_transform = transforms[gl_InstanceID];
    vec4 vertex_ws = instance_transform * vec4(Vertex_Position, 1.0);

    gl_Position = projection * view * vertex_ws;
    instance_id = 0; //used for color, so put 0 here for now...
    vertex_color = vertex_color;
    vertex_position_ws = vertex_ws.xyz;
    vertex_normal_ws = (instance_transform * vec4(Vertex_Normal, 0.0)).xyz;

    curve_position_ws =vec3(0.0, 0.0, 0.0); // this is only used for discarding fragments, which is irrelevant for the arch
    
} 

