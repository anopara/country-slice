#version 450 core

layout (location = 0) in vec3 Vertex_Position;   // the position variable has attribute position 0
layout (location = 1) in vec3 Vertex_Color; // the color variable has attribute position 1
  
out vec3 ourColor; // output a color to the fragment shader

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout (std430, binding=2) buffer transforms_buffer { 
    mat4 transforms[];
};


float random( int p ) {
    return fract(sin(dot(float(p), 311.7)) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main()
{   
    //vec3 final_p = Vertex_Position + vec3(0.0, gl_InstanceID, 0.0);
    //vec3 final_p =
    //gl_Position = projection * view * model * final_p;//vec4(final_p, 1.0);
    gl_Position = projection * view * model * transforms[gl_InstanceID] * vec4(Vertex_Position, 1.0);
    //ourColor = Vertex_Color; 
    float r = fit01(random(gl_InstanceID),0.1, 0.5);// 0.2, 0.25);
    ourColor = vec3(r);
} 