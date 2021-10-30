#version 450
layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

// Custom attribs
layout(location = 1) in int Instance_Id;
layout(location = 2) in vec2 Curve_Uv_Pos;

layout(location = 0) out vec3 v_color;

float random_f(float x) {
    return fract(sin(x*12.9898) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main() {

    //const float WALL_HEIGHT = 1.4;
    //const float SEED = 12312.0;

    // Add wavery pattern

    //float bby = Vertex_Position.y / WALL_HEIGHT; 
//
    //float amp = fit01(random_f(bby*1000.0+SEED), 0.5, 1.5);
//
    //float sin_wave = sin()
//
    //vec3 pos = vec3(Vertex_Position.x,   , Vertex_Position.z);



    // OUT-------------

    //float v = fit01(random_f(float(Instance_Id)), 0.3, 0.65);
    v_color = vec3(Curve_Uv_Pos, 0.0); //vec3(v,v,v);

    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}

