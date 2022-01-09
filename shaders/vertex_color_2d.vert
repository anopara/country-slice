#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; // the color variable has attribute position 1
  
out vec3 ourColor; // output a color to the fragment shader

uniform vec2 window_size;


void main()
{   

    vec2 ss_pos = Vertex_Position.xy;

    vec2 ndc = ss_pos / window_size * 2.0 - vec2(1.0, 1.0);
    ndc.y = - ndc.y;

    gl_Position = vec4(ndc, 0.0, 1.0);
    ourColor = Vertex_Color; // set ourColor to the input color we got from the vertex data
} 