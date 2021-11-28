#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; // the color variable has attribute position 1
layout (location = 2) in vec2 Vertex_UV;
  
out vec3 ourColor; // output a color to the fragment shader
out vec2 TexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{   
    gl_Position = projection * view * model * vec4(Vertex_Position, 1.0);
    ourColor = Vertex_Color; // set ourColor to the input color we got from the vertex data
    TexCoord = Vertex_UV;
} 